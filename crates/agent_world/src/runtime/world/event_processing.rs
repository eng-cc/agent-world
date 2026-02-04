use super::World;
use super::super::{
    Action, ActionEnvelope, CausedBy, DomainEvent, RejectReason, WorldError, WorldEvent,
    WorldEventBody, WorldEventId, WorldTime,
};

impl World {
    // ---------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------

    pub(super) fn replay_from(&mut self, start_index: usize) -> Result<(), WorldError> {
        let events: Vec<WorldEvent> = self.journal.events[start_index..].to_vec();
        for event in events {
            self.apply_event_body(&event.body, event.time)?;
            self.state.time = event.time;
            self.next_event_id = self.next_event_id.max(event.id.saturating_add(1));
        }
        Ok(())
    }

    pub(super) fn action_to_event(
        &self,
        envelope: &ActionEnvelope,
    ) -> Result<WorldEventBody, WorldError> {
        let action_id = envelope.id;
        match &envelope.action {
            Action::RegisterAgent { agent_id, pos } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentAlreadyExists {
                            agent_id: agent_id.clone(),
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::AgentRegistered {
                        agent_id: agent_id.clone(),
                        pos: *pos,
                    }))
                }
            }
            Action::MoveAgent { agent_id, to } => match self.state.agents.get(agent_id) {
                Some(cell) => Ok(WorldEventBody::Domain(DomainEvent::AgentMoved {
                    agent_id: agent_id.clone(),
                    from: cell.state.pos,
                    to: *to,
                })),
                None => Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    },
                })),
            },
        }
    }

    pub(super) fn append_event(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        self.apply_event_body(&body, self.state.time)?;
        let event_id = self.next_event_id;
        self.next_event_id += 1;
        self.journal.append(WorldEvent {
            id: event_id,
            time: self.state.time,
            caused_by,
            body,
        });
        Ok(event_id)
    }

    fn apply_event_body(&mut self, body: &WorldEventBody, time: WorldTime) -> Result<(), WorldError> {
        match body {
            WorldEventBody::Domain(event) => {
                self.state.apply_domain_event(event, time)?;
                self.state.route_domain_event(event);
            }
            WorldEventBody::EffectQueued(intent) => {
                self.pending_effects.push_back(intent.clone());
            }
            WorldEventBody::ReceiptAppended(receipt) => {
                let mut removed = false;
                if self.inflight_effects.remove(&receipt.intent_id).is_some() {
                    removed = true;
                }
                let before = self.pending_effects.len();
                self.pending_effects
                    .retain(|intent| intent.intent_id != receipt.intent_id);
                if before != self.pending_effects.len() {
                    removed = true;
                }
                if !removed {
                    return Err(WorldError::ReceiptUnknownIntent {
                        intent_id: receipt.intent_id.clone(),
                    });
                }
            }
            WorldEventBody::PolicyDecisionRecorded(_) => {}
            WorldEventBody::Governance(event) => {
                self.apply_governance_event(event)?;
            }
            WorldEventBody::ModuleEvent(event) => {
                self.apply_module_event(event, time)?;
            }
            WorldEventBody::ModuleCallFailed(_) => {}
            WorldEventBody::ModuleEmitted(_) => {}
            WorldEventBody::ModuleStateUpdated(update) => {
                self.state
                    .module_states
                    .insert(update.module_id.clone(), update.state.clone());
            }
            WorldEventBody::SnapshotCreated(_) => {}
            WorldEventBody::ManifestUpdated(update) => {
                self.manifest = update.manifest.clone();
            }
            WorldEventBody::RollbackApplied(_) => {}
        }
        self.state.time = time;
        Ok(())
    }
}
