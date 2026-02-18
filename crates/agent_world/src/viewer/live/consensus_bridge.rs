use super::*;
use crate::consensus_action_payload::{
    decode_consensus_action_payload, encode_consensus_action_payload, ConsensusActionPayloadBody,
    ConsensusActionPayloadEnvelope,
};
use agent_world_node::NodeRuntime;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_INFLIGHT_CONSENSUS_ACTIONS: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CommittedLiveAction {
    action: Action,
    submitter: ActionSubmitter,
}

#[derive(Debug)]
pub(super) struct LiveConsensusBridge {
    runtime: Arc<Mutex<NodeRuntime>>,
    next_action_id: u64,
    inflight_action_ids: HashSet<u64>,
    committed_actions: VecDeque<CommittedLiveAction>,
}

impl LiveConsensusBridge {
    pub(super) fn new(runtime: Arc<Mutex<NodeRuntime>>) -> Self {
        Self {
            runtime,
            next_action_id: initial_consensus_action_id_seed(),
            inflight_action_ids: HashSet::new(),
            committed_actions: VecDeque::new(),
        }
    }

    pub(super) fn reset_pending(&mut self) {
        self.inflight_action_ids.clear();
        self.committed_actions.clear();
    }

    pub(super) fn has_inflight_capacity(&self) -> bool {
        self.inflight_action_ids.len() < MAX_INFLIGHT_CONSENSUS_ACTIONS
    }

    pub(super) fn refresh_committed_actions(&mut self) -> Result<(), ViewerLiveServerError> {
        let batches = self
            .runtime
            .lock()
            .map_err(|_| {
                ViewerLiveServerError::Node(
                    "viewer live consensus bridge: node runtime lock poisoned".to_string(),
                )
            })?
            .drain_committed_action_batches();

        for batch in batches {
            for committed in batch.actions {
                self.inflight_action_ids.remove(&committed.action_id);
                match decode_consensus_action_payload(committed.payload_cbor.as_slice()) {
                    Ok(ConsensusActionPayloadBody::SimulatorAction { action, submitter }) => {
                        self.committed_actions
                            .push_back(CommittedLiveAction { action, submitter });
                    }
                    Ok(ConsensusActionPayloadBody::RuntimeAction { .. }) => {}
                    Err(err) => {
                        eprintln!(
                            "viewer live consensus bridge: skip undecodable payload action_id={} err={}",
                            committed.action_id, err
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub(super) fn pop_committed_action(&mut self) -> Option<CommittedLiveAction> {
        self.committed_actions.pop_front()
    }

    pub(super) fn submit_action(
        &mut self,
        action: Action,
        submitter: ActionSubmitter,
    ) -> Result<(), ViewerLiveServerError> {
        let action_id = self.next_action_id.max(1);
        self.next_action_id = action_id.saturating_add(1).max(1);

        let payload = encode_consensus_action_payload(
            &ConsensusActionPayloadEnvelope::from_simulator_action(action, submitter),
        )
        .map_err(ViewerLiveServerError::Serde)?;

        self.runtime
            .lock()
            .map_err(|_| {
                ViewerLiveServerError::Node(
                    "viewer live consensus bridge: node runtime lock poisoned".to_string(),
                )
            })?
            .submit_consensus_action_payload(action_id, payload)
            .map_err(|err| {
                ViewerLiveServerError::Node(format!(
                    "viewer live consensus bridge submit failed action_id={action_id}: {err:?}"
                ))
            })?;
        self.inflight_action_ids.insert(action_id);
        Ok(())
    }
}

impl LiveWorld {
    pub(super) fn step_via_consensus(&mut self) -> Result<LiveStepResult, ViewerLiveServerError> {
        if let Some(bridge) = self.consensus_bridge.as_mut() {
            bridge.refresh_committed_actions()?;
            if let Some(committed) = bridge.pop_committed_action() {
                let event = self.apply_committed_consensus_action(committed)?;
                return Ok(LiveStepResult {
                    event: Some(event),
                    decision_trace: None,
                });
            }
            if !bridge.has_inflight_capacity() {
                return Ok(LiveStepResult {
                    event: None,
                    decision_trace: None,
                });
            }
        }

        match &mut self.driver {
            LiveDriver::Script(script) => {
                if let Some(action) = script.next_action(&self.kernel) {
                    if let Some(bridge) = self.consensus_bridge.as_mut() {
                        bridge.submit_action(action, ActionSubmitter::System)?;
                    }
                }
                Ok(LiveStepResult {
                    event: None,
                    decision_trace: None,
                })
            }
            LiveDriver::Llm(runner) => {
                let tick_result = runner.tick_decide_only(&mut self.kernel);
                sync_llm_runner_long_term_memory(&mut self.kernel, runner);
                let mut decision_trace = None;
                if let Some(result) = tick_result {
                    decision_trace = result.decision_trace;
                    if let AgentDecision::Act(action) = result.decision {
                        if let Some(bridge) = self.consensus_bridge.as_mut() {
                            bridge.submit_action(
                                action,
                                ActionSubmitter::Agent {
                                    agent_id: result.agent_id,
                                },
                            )?;
                        }
                    }
                }
                Ok(LiveStepResult {
                    event: None,
                    decision_trace,
                })
            }
        }
    }

    fn apply_committed_consensus_action(
        &mut self,
        committed: CommittedLiveAction,
    ) -> Result<WorldEvent, ViewerLiveServerError> {
        let action = committed.action;
        let submitter = committed.submitter;
        let action_id = match &submitter {
            ActionSubmitter::System => self.kernel.submit_action_from_system(action.clone()),
            ActionSubmitter::Agent { agent_id } => self
                .kernel
                .submit_action_from_agent(agent_id.clone(), action.clone()),
            ActionSubmitter::Player { player_id } => self
                .kernel
                .submit_action_from_player(player_id.clone(), action.clone()),
        };
        let event = self.kernel.step().ok_or_else(|| {
            ViewerLiveServerError::Node(
                "viewer live consensus bridge: kernel step produced no event".to_string(),
            )
        })?;

        if let LiveDriver::Llm(runner) = &mut self.driver {
            if let ActionSubmitter::Agent { agent_id } = &submitter {
                let success = !matches!(event.kind, WorldEventKind::ActionRejected { .. });
                let action_result = ActionResult {
                    action,
                    action_id,
                    success,
                    event: event.clone(),
                };
                let _ = runner.notify_action_result(agent_id.as_str(), &action_result);
            }
            sync_llm_runner_long_term_memory(&mut self.kernel, runner);
        }

        Ok(event)
    }
}

fn initial_consensus_action_id_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1)
        .max(1)
}
