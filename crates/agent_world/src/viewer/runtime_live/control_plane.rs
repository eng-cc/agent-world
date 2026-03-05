use super::*;
use std::collections::BTreeMap;

use super::super::auth::{
    verify_agent_chat_auth_proof, verify_prompt_control_apply_auth_proof,
    verify_prompt_control_rollback_auth_proof, PromptControlAuthIntent,
};
use super::super::protocol::{
    AgentChatAck, AgentChatError, AgentChatRequest, PromptControlAck, PromptControlApplyRequest,
    PromptControlCommand, PromptControlError, PromptControlOperation, PromptControlRollbackRequest,
};
use crate::runtime::{
    Action as RuntimeAction, CausedBy as RuntimeCausedBy, ModuleSourcePackage,
    World as RuntimeWorld,
};
use crate::simulator::{
    Action as SimulatorAction, ActionResult, AgentDecision, AgentDecisionTrace, AgentPromptProfile,
    AgentRunner, LlmAgentBehavior, OpenAiChatCompletionClient, PromptUpdateOperation,
    ResourceOwner, WorldEventKind, WorldJournal, WorldKernel, WorldSnapshot,
};
use sha2::{Digest, Sha256};

impl ViewerRuntimeLiveServer {
    pub(super) fn handle_prompt_control(
        &mut self,
        command: PromptControlCommand,
    ) -> Result<PromptControlAck, PromptControlError> {
        if !self.llm_sidecar.is_llm_mode() {
            let (agent_id, message) = match command {
                PromptControlCommand::Preview { request }
                | PromptControlCommand::Apply { request } => (
                    request.agent_id,
                    "prompt_control requires runtime live server running with --llm".to_string(),
                ),
                PromptControlCommand::Rollback { request } => (
                    request.agent_id,
                    "prompt_control rollback requires runtime live server running with --llm"
                        .to_string(),
                ),
            };
            return Err(PromptControlError {
                code: "llm_mode_required".to_string(),
                message,
                agent_id: Some(agent_id.clone()),
                current_version: self.current_prompt_version(agent_id.as_str()),
            });
        }

        match command {
            PromptControlCommand::Preview { request } => self.prompt_control_preview(request),
            PromptControlCommand::Apply { request } => self.prompt_control_apply(request),
            PromptControlCommand::Rollback { request } => self.prompt_control_rollback(request),
        }
    }

    fn prompt_control_preview(
        &mut self,
        request: PromptControlApplyRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let player_id =
            normalize_required_player_id(request.player_id.as_str(), request.agent_id.as_str())?;
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        self.verify_and_consume_prompt_control_apply_auth(
            PromptControlAuthIntent::Preview,
            &request,
        )?;
        ensure_agent_player_access_runtime(
            &self.world,
            &self.llm_sidecar,
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version_runtime(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;

        let mut candidate = current.clone();
        apply_prompt_patch_runtime(&mut candidate, &request);
        let applied_fields = changed_prompt_fields_runtime(&current, &candidate);
        let preview_version = if applied_fields.is_empty() {
            current.version
        } else {
            current.version.saturating_add(1)
        };

        Ok(PromptControlAck {
            agent_id: request.agent_id,
            operation: PromptControlOperation::Apply,
            preview: true,
            version: preview_version,
            updated_at_tick: self.world.state().time,
            applied_fields,
            digest: prompt_profile_digest_runtime(&candidate),
            rolled_back_to_version: None,
        })
    }

    fn prompt_control_apply(
        &mut self,
        request: PromptControlApplyRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let player_id =
            normalize_required_player_id(request.player_id.as_str(), request.agent_id.as_str())?;
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        self.verify_and_consume_prompt_control_apply_auth(
            PromptControlAuthIntent::Apply,
            &request,
        )?;
        ensure_agent_player_access_runtime(
            &self.world,
            &self.llm_sidecar,
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version_runtime(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;
        ensure_updated_by_matches_player_runtime(
            request.updated_by.as_deref(),
            player_id.as_str(),
            request.agent_id.as_str(),
        )?;

        let mut candidate = current.clone();
        apply_prompt_patch_runtime(&mut candidate, &request);
        let applied_fields = changed_prompt_fields_runtime(&current, &candidate);
        let digest = prompt_profile_digest_runtime(&candidate);
        if applied_fields.is_empty() {
            return Ok(PromptControlAck {
                agent_id: request.agent_id,
                operation: PromptControlOperation::Apply,
                preview: false,
                version: current.version,
                updated_at_tick: current.updated_at_tick,
                applied_fields,
                digest,
                rolled_back_to_version: None,
            });
        }

        candidate.version = current.version.saturating_add(1);
        candidate.updated_at_tick = self.world.state().time;
        candidate.updated_by = player_id.clone();
        self.llm_sidecar.upsert_prompt_profile(candidate.clone());
        self.llm_sidecar.apply_prompt_profile_to_driver(&candidate);
        self.bind_agent_player_access(
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let digest = prompt_profile_digest_runtime(&candidate);
        self.enqueue_virtual_event(WorldEventKind::AgentPromptUpdated {
            profile: candidate.clone(),
            operation: PromptUpdateOperation::Apply,
            applied_fields: applied_fields.clone(),
            digest: digest.clone(),
            rolled_back_to_version: None,
        });
        self.llm_sidecar.request_decision();

        Ok(PromptControlAck {
            agent_id: request.agent_id,
            operation: PromptControlOperation::Apply,
            preview: false,
            version: candidate.version,
            updated_at_tick: candidate.updated_at_tick,
            applied_fields,
            digest,
            rolled_back_to_version: None,
        })
    }

    fn prompt_control_rollback(
        &mut self,
        request: PromptControlRollbackRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let player_id =
            normalize_required_player_id(request.player_id.as_str(), request.agent_id.as_str())?;
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        self.verify_and_consume_prompt_control_rollback_auth(&request)?;
        ensure_agent_player_access_runtime(
            &self.world,
            &self.llm_sidecar,
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version_runtime(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;
        ensure_updated_by_matches_player_runtime(
            request.updated_by.as_deref(),
            player_id.as_str(),
            request.agent_id.as_str(),
        )?;

        let target = if request.to_version == 0 {
            AgentPromptProfile::for_agent(request.agent_id.clone())
        } else {
            self.lookup_prompt_profile_version(request.agent_id.as_str(), request.to_version)
                .ok_or_else(|| PromptControlError {
                    code: "target_version_not_found".to_string(),
                    message: format!(
                        "prompt profile version {} not found for {}",
                        request.to_version, request.agent_id
                    ),
                    agent_id: Some(request.agent_id.clone()),
                    current_version: Some(current.version),
                })?
        };

        let mut candidate = current.clone();
        candidate.system_prompt_override = target.system_prompt_override;
        candidate.short_term_goal_override = target.short_term_goal_override;
        candidate.long_term_goal_override = target.long_term_goal_override;
        let applied_fields = changed_prompt_fields_runtime(&current, &candidate);
        if applied_fields.is_empty() {
            return Err(PromptControlError {
                code: "rollback_noop".to_string(),
                message: format!(
                    "rollback target version {} yields no prompt changes for {}",
                    request.to_version, request.agent_id
                ),
                agent_id: Some(request.agent_id),
                current_version: Some(current.version),
            });
        }

        candidate.version = current.version.saturating_add(1);
        candidate.updated_at_tick = self.world.state().time;
        candidate.updated_by = player_id.clone();
        self.llm_sidecar.upsert_prompt_profile(candidate.clone());
        self.llm_sidecar.apply_prompt_profile_to_driver(&candidate);
        self.bind_agent_player_access(
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let digest = prompt_profile_digest_runtime(&candidate);
        self.enqueue_virtual_event(WorldEventKind::AgentPromptUpdated {
            profile: candidate.clone(),
            operation: PromptUpdateOperation::Rollback,
            applied_fields: applied_fields.clone(),
            digest: digest.clone(),
            rolled_back_to_version: Some(request.to_version),
        });
        self.llm_sidecar.request_decision();

        Ok(PromptControlAck {
            agent_id: request.agent_id,
            operation: PromptControlOperation::Rollback,
            preview: false,
            version: candidate.version,
            updated_at_tick: candidate.updated_at_tick,
            applied_fields,
            digest,
            rolled_back_to_version: Some(request.to_version),
        })
    }

    pub(super) fn handle_agent_chat(
        &mut self,
        request: AgentChatRequest,
    ) -> Result<AgentChatAck, AgentChatError> {
        if !self.llm_sidecar.is_llm_mode() {
            return Err(AgentChatError {
                code: "llm_mode_required".to_string(),
                message: "agent chat requires runtime live server running with --llm".to_string(),
                agent_id: Some(request.agent_id),
            });
        }

        let player_id = request
            .player_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let Some(player_id) = player_id else {
            return Err(AgentChatError {
                code: "player_id_required".to_string(),
                message: "agent_chat requires non-empty player_id".to_string(),
                agent_id: Some(request.agent_id),
            });
        };
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        let message = request.message.trim().to_string();
        if message.is_empty() {
            return Err(AgentChatError {
                code: "empty_message".to_string(),
                message: "chat message cannot be empty".to_string(),
                agent_id: Some(request.agent_id),
            });
        }
        self.verify_and_consume_agent_chat_auth(&request)?;
        self.bind_agent_player_access_for_chat(
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        self.llm_sidecar.push_chat_message(
            &self.world,
            &self.snapshot_config,
            request.agent_id.as_str(),
            message.as_str(),
        )?;
        self.llm_sidecar.request_decision();
        Ok(AgentChatAck {
            agent_id: request.agent_id,
            accepted_at_tick: self.world.state().time,
            message_len: request.message.trim().chars().count(),
            player_id: Some(player_id),
        })
    }

    fn verify_and_consume_prompt_control_apply_auth(
        &mut self,
        intent: PromptControlAuthIntent,
        request: &PromptControlApplyRequest,
    ) -> Result<(), PromptControlError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(PromptControlError {
                code: "auth_proof_required".to_string(),
                message: "prompt_control requires auth proof".to_string(),
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            });
        };
        let verified =
            verify_prompt_control_apply_auth_proof(intent, request, auth).map_err(|message| {
                PromptControlError {
                    code: map_auth_verify_error_code(message.as_str()).to_string(),
                    message,
                    agent_id: Some(request.agent_id.clone()),
                    current_version: self.current_prompt_version(request.agent_id.as_str()),
                }
            })?;
        self.llm_sidecar
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| PromptControlError {
                code: "auth_nonce_replay".to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            })?;
        Ok(())
    }

    fn verify_and_consume_prompt_control_rollback_auth(
        &mut self,
        request: &PromptControlRollbackRequest,
    ) -> Result<(), PromptControlError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(PromptControlError {
                code: "auth_proof_required".to_string(),
                message: "prompt_control rollback requires auth proof".to_string(),
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            });
        };
        let verified =
            verify_prompt_control_rollback_auth_proof(request, auth).map_err(|message| {
                PromptControlError {
                    code: map_auth_verify_error_code(message.as_str()).to_string(),
                    message,
                    agent_id: Some(request.agent_id.clone()),
                    current_version: self.current_prompt_version(request.agent_id.as_str()),
                }
            })?;
        self.llm_sidecar
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| PromptControlError {
                code: "auth_nonce_replay".to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            })?;
        Ok(())
    }

    fn verify_and_consume_agent_chat_auth(
        &mut self,
        request: &AgentChatRequest,
    ) -> Result<(), AgentChatError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(AgentChatError {
                code: "auth_proof_required".to_string(),
                message: "agent_chat requires auth proof".to_string(),
                agent_id: Some(request.agent_id.clone()),
            });
        };
        let verified =
            verify_agent_chat_auth_proof(request, auth).map_err(|message| AgentChatError {
                code: map_auth_verify_error_code(message.as_str()).to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
            })?;
        self.llm_sidecar
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| AgentChatError {
                code: "auth_nonce_replay".to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
            })?;
        Ok(())
    }

    fn current_prompt_version(&self, agent_id: &str) -> Option<u64> {
        self.llm_sidecar
            .prompt_profiles
            .get(agent_id)
            .map(|profile| profile.version)
    }

    fn current_prompt_profile(
        &self,
        agent_id: &str,
    ) -> Result<AgentPromptProfile, PromptControlError> {
        if !self.world.state().agents.contains_key(agent_id) {
            return Err(PromptControlError {
                code: "agent_not_found".to_string(),
                message: format!("agent not found: {agent_id}"),
                agent_id: Some(agent_id.to_string()),
                current_version: None,
            });
        }
        Ok(self
            .llm_sidecar
            .prompt_profiles
            .get(agent_id)
            .cloned()
            .unwrap_or_else(|| AgentPromptProfile::for_agent(agent_id.to_string())))
    }

    fn lookup_prompt_profile_version(
        &self,
        agent_id: &str,
        version: u64,
    ) -> Option<AgentPromptProfile> {
        self.llm_sidecar
            .prompt_profile_history
            .get(agent_id)
            .and_then(|versions| versions.get(&version).cloned())
            .or_else(|| {
                let profile = self.llm_sidecar.prompt_profiles.get(agent_id)?;
                if profile.version == version {
                    Some(profile.clone())
                } else {
                    None
                }
            })
    }

    fn bind_agent_player_access(
        &mut self,
        agent_id: &str,
        player_id: &str,
        public_key: Option<&str>,
    ) -> Result<(), PromptControlError> {
        ensure_agent_player_access_runtime(
            &self.world,
            &self.llm_sidecar,
            agent_id,
            player_id,
            public_key,
        )?;
        if let Some(event) = self
            .llm_sidecar
            .bind_agent_player(agent_id, player_id, public_key)
            .map_err(|message| PromptControlError {
                code: "player_bind_failed".to_string(),
                message,
                agent_id: Some(agent_id.to_string()),
                current_version: self.current_prompt_version(agent_id),
            })?
        {
            self.enqueue_virtual_event(event);
        }
        Ok(())
    }

    fn bind_agent_player_access_for_chat(
        &mut self,
        agent_id: &str,
        player_id: &str,
        public_key: Option<&str>,
    ) -> Result<(), AgentChatError> {
        let mapped = ensure_agent_player_access_runtime(
            &self.world,
            &self.llm_sidecar,
            agent_id,
            player_id,
            public_key,
        )
        .map_err(|err| AgentChatError {
            code: "agent_control_forbidden".to_string(),
            message: err.message,
            agent_id: err.agent_id,
        });
        mapped?;
        if let Some(event) = self
            .llm_sidecar
            .bind_agent_player(agent_id, player_id, public_key)
            .map_err(|message| AgentChatError {
                code: "player_bind_failed".to_string(),
                message,
                agent_id: Some(agent_id.to_string()),
            })?
        {
            self.enqueue_virtual_event(event);
        }
        Ok(())
    }

    fn enqueue_virtual_event(&mut self, kind: WorldEventKind) {
        let id = self.next_virtual_event_id();
        self.pending_virtual_events.push_back(WorldEvent {
            id,
            time: self.world.state().time,
            kind,
        });
    }

    fn next_virtual_event_id(&mut self) -> u64 {
        let floor = latest_runtime_event_seq(&self.world)
            .saturating_add(1)
            .max(1);
        if self.next_virtual_event_id < floor {
            self.next_virtual_event_id = floor;
        }
        let id = self.next_virtual_event_id;
        self.next_virtual_event_id = self.next_virtual_event_id.saturating_add(1);
        id
    }

    pub(super) fn enqueue_llm_action_from_sidecar(&mut self) -> Option<AgentDecisionTrace> {
        let Some(decision) = self
            .llm_sidecar
            .next_llm_decision(&self.world, &self.snapshot_config)
        else {
            return None;
        };
        let decision_trace = decision.decision_trace.clone();
        if let Some(trace) = decision_trace.as_ref() {
            if let Some(message) = trace
                .llm_error
                .as_ref()
                .or_else(|| trace.parse_error.as_ref())
            {
                self.enqueue_virtual_event(WorldEventKind::ActionRejected {
                    reason: SimulatorRejectReason::RuleDenied {
                        notes: vec![format!("llm_failed: {}", message)],
                    },
                });
                return decision_trace;
            }
        }

        if let AgentDecision::Act(action) = decision.decision {
            match simulator_action_to_runtime(&action, &self.world) {
                Some(runtime_action) => {
                    let action_id = self.world.submit_action(runtime_action);
                    self.llm_sidecar
                        .track_action(action_id, decision.agent_id, action.clone());
                }
                None => {
                    self.enqueue_virtual_event(WorldEventKind::ActionRejected {
                        reason: SimulatorRejectReason::RuleDenied {
                            notes: vec![format!(
                                "runtime llm bridge cannot map action: {}",
                                simulator_action_label(&action)
                            )],
                        },
                    });
                }
            }
        }
        decision_trace
    }
}

pub(super) struct RuntimeLlmSidecar {
    pub(super) decision_mode: ViewerLiveDecisionMode,
    pub(super) prompt_profiles: BTreeMap<String, AgentPromptProfile>,
    pub(super) prompt_profile_history: BTreeMap<String, BTreeMap<u64, AgentPromptProfile>>,
    pub(super) agent_player_bindings: BTreeMap<String, String>,
    pub(super) agent_public_key_bindings: BTreeMap<String, String>,
    pub(super) player_auth_last_nonce: BTreeMap<String, u64>,
    llm_decision_mailbox: u64,
    runner: Option<AgentRunner<LlmAgentBehavior<OpenAiChatCompletionClient>>>,
    shadow_kernel: Option<WorldKernel>,
    pending_actions: BTreeMap<u64, RuntimePendingAction>,
}

impl RuntimeLlmSidecar {
    pub(super) fn new(decision_mode: ViewerLiveDecisionMode) -> Self {
        Self {
            decision_mode,
            prompt_profiles: BTreeMap::new(),
            prompt_profile_history: BTreeMap::new(),
            agent_player_bindings: BTreeMap::new(),
            agent_public_key_bindings: BTreeMap::new(),
            player_auth_last_nonce: BTreeMap::new(),
            llm_decision_mailbox: 0,
            runner: None,
            shadow_kernel: None,
            pending_actions: BTreeMap::new(),
        }
    }

    pub(super) fn is_llm_mode(&self) -> bool {
        matches!(self.decision_mode, ViewerLiveDecisionMode::Llm)
    }

    pub(super) fn consume_player_auth_nonce(
        &mut self,
        player_id: &str,
        nonce: u64,
    ) -> Result<(), String> {
        let player_id = player_id.trim();
        if player_id.is_empty() {
            return Err("player_id cannot be empty".to_string());
        }
        if nonce == 0 {
            return Err("auth nonce must be greater than zero".to_string());
        }
        if let Some(last_nonce) = self.player_auth_last_nonce.get(player_id) {
            if nonce <= *last_nonce {
                return Err(format!(
                    "auth nonce replay for {}: expected nonce > {}, received {}",
                    player_id, last_nonce, nonce
                ));
            }
        }
        self.player_auth_last_nonce
            .insert(player_id.to_string(), nonce);
        Ok(())
    }

    pub(super) fn bind_agent_player(
        &mut self,
        agent_id: &str,
        player_id: &str,
        public_key: Option<&str>,
    ) -> Result<Option<WorldEventKind>, String> {
        let player_id = player_id.trim();
        if player_id.is_empty() {
            return Err("player_id cannot be empty".to_string());
        }
        let requested_public_key = public_key
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let current_player = self.agent_player_bindings.get(agent_id).cloned();
        let current_public_key = self.agent_public_key_bindings.get(agent_id).cloned();
        let target_public_key = if current_player.as_deref() == Some(player_id) {
            requested_public_key
                .clone()
                .or_else(|| current_public_key.clone())
        } else {
            requested_public_key.clone()
        };
        if current_player.as_deref() == Some(player_id) && current_public_key == target_public_key {
            return Ok(None);
        }

        self.agent_player_bindings
            .insert(agent_id.to_string(), player_id.to_string());
        match target_public_key.clone() {
            Some(value) => {
                self.agent_public_key_bindings
                    .insert(agent_id.to_string(), value);
            }
            None => {
                self.agent_public_key_bindings.remove(agent_id);
            }
        }
        Ok(Some(WorldEventKind::AgentPlayerBound {
            agent_id: agent_id.to_string(),
            player_id: player_id.to_string(),
            public_key: target_public_key,
        }))
    }

    pub(super) fn upsert_prompt_profile(&mut self, profile: AgentPromptProfile) {
        self.prompt_profile_history
            .entry(profile.agent_id.clone())
            .or_default()
            .insert(profile.version, profile.clone());
        self.prompt_profiles
            .insert(profile.agent_id.clone(), profile);
    }

    pub(super) fn request_decision(&mut self) {
        if self.is_llm_mode() {
            self.llm_decision_mailbox = self.llm_decision_mailbox.saturating_add(1);
        }
    }

    pub(super) fn apply_prompt_profile_to_driver(&mut self, profile: &AgentPromptProfile) {
        let Some(runner) = self.runner.as_mut() else {
            return;
        };
        let Some(agent) = runner.get_mut(profile.agent_id.as_str()) else {
            return;
        };
        agent.behavior.apply_prompt_overrides(
            profile.system_prompt_override.clone(),
            profile.short_term_goal_override.clone(),
            profile.long_term_goal_override.clone(),
        );
    }

    pub(super) fn push_chat_message(
        &mut self,
        world: &RuntimeWorld,
        config: &WorldConfig,
        agent_id: &str,
        message: &str,
    ) -> Result<(), AgentChatError> {
        if !self.is_llm_mode() {
            return Err(AgentChatError {
                code: "llm_mode_required".to_string(),
                message: "agent chat requires runtime live server running with --llm".to_string(),
                agent_id: Some(agent_id.to_string()),
            });
        }
        if let Err(message) = self.sync_shadow_kernel(world, config) {
            return Err(AgentChatError {
                code: "llm_init_failed".to_string(),
                message,
                agent_id: Some(agent_id.to_string()),
            });
        }
        if let Err(message) = self.ensure_runner_initialized() {
            return Err(AgentChatError {
                code: "llm_init_failed".to_string(),
                message,
                agent_id: Some(agent_id.to_string()),
            });
        }
        let runner = match self.runner.as_mut() {
            Some(runner) => runner,
            None => {
                return Err(AgentChatError {
                    code: "llm_init_failed".to_string(),
                    message: "llm runner not initialized".to_string(),
                    agent_id: Some(agent_id.to_string()),
                });
            }
        };
        let Some(agent) = runner.get_mut(agent_id) else {
            return Err(AgentChatError {
                code: "agent_not_registered".to_string(),
                message: format!("agent {} is not registered in llm runner", agent_id),
                agent_id: Some(agent_id.to_string()),
            });
        };
        if !agent
            .behavior
            .push_player_message(world.state().time, message)
        {
            return Err(AgentChatError {
                code: "empty_message".to_string(),
                message: "chat message cannot be empty".to_string(),
                agent_id: Some(agent_id.to_string()),
            });
        }
        Ok(())
    }

    pub(super) fn next_llm_decision(
        &mut self,
        world: &RuntimeWorld,
        config: &WorldConfig,
    ) -> Option<RuntimeLlmDecision> {
        if !self.is_llm_mode() || self.llm_decision_mailbox == 0 {
            return None;
        }
        self.llm_decision_mailbox = self.llm_decision_mailbox.saturating_sub(1);

        if let Err(message) = self.sync_shadow_kernel(world, config) {
            return Some(RuntimeLlmDecision::from_error(world, message));
        }
        if let Err(message) = self.ensure_runner_initialized() {
            return Some(RuntimeLlmDecision::from_error(world, message));
        }
        let kernel = match self.shadow_kernel.as_mut() {
            Some(kernel) => kernel,
            None => {
                return Some(RuntimeLlmDecision::from_error(
                    world,
                    "shadow kernel not initialized".to_string(),
                ));
            }
        };
        let runner = match self.runner.as_mut() {
            Some(runner) => runner,
            None => {
                return Some(RuntimeLlmDecision::from_error(
                    world,
                    "llm runner not initialized".to_string(),
                ));
            }
        };
        let result = runner.tick_decide_only(kernel);
        sync_llm_runner_long_term_memory(kernel, runner);
        result.map(|tick| RuntimeLlmDecision {
            agent_id: tick.agent_id,
            decision: tick.decision,
            decision_trace: tick.decision_trace,
        })
    }

    pub(super) fn track_action(
        &mut self,
        action_id: u64,
        agent_id: String,
        action: SimulatorAction,
    ) {
        self.pending_actions
            .insert(action_id, RuntimePendingAction { agent_id, action });
    }

    pub(super) fn notify_action_result(
        &mut self,
        action_id: u64,
        event: WorldEvent,
        rejected: bool,
    ) {
        let Some(pending) = self.pending_actions.remove(&action_id) else {
            return;
        };
        let success = !rejected;
        let action_result = ActionResult {
            action: pending.action,
            action_id,
            success,
            event,
        };
        if let Some(runner) = self.runner.as_mut() {
            let _ = runner.notify_action_result(pending.agent_id.as_str(), &action_result);
        }
    }

    pub(super) fn notify_action_result_if_needed(
        &mut self,
        runtime_event: &RuntimeWorldEvent,
        mapped_event: WorldEvent,
    ) {
        let Some(caused_by) = runtime_event.caused_by.as_ref() else {
            return;
        };
        let RuntimeCausedBy::Action(action_id) = caused_by else {
            return;
        };
        let rejected = matches!(
            runtime_event.body,
            RuntimeWorldEventBody::Domain(RuntimeDomainEvent::ActionRejected { .. })
        );
        self.notify_action_result(*action_id, mapped_event, rejected);
    }

    fn sync_shadow_kernel(
        &mut self,
        world: &RuntimeWorld,
        config: &WorldConfig,
    ) -> Result<(), String> {
        let runtime_snapshot = world.snapshot();
        let model = runtime_state_to_simulator_model(world.state(), self);
        let snapshot = WorldSnapshot {
            version: SNAPSHOT_VERSION,
            chunk_generation_schema_version: CHUNK_GENERATION_SCHEMA_VERSION,
            time: world.state().time,
            config: config.clone(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: runtime_snapshot.last_event_id.saturating_add(1).max(1),
            next_action_id: runtime_snapshot.next_action_id.max(1),
            pending_actions: Vec::new(),
            journal_len: 0,
        };
        let kernel = WorldKernel::from_snapshot(snapshot, WorldJournal::new())
            .map_err(|err| format!("runtime live shadow kernel rebuild failed: {err:?}"))?;
        self.shadow_kernel = Some(kernel);
        Ok(())
    }

    fn ensure_runner_initialized(&mut self) -> Result<(), String> {
        let kernel = self
            .shadow_kernel
            .as_ref()
            .ok_or_else(|| "shadow kernel not initialized".to_string())?;
        if self.runner.is_none() {
            self.runner = Some(AgentRunner::new());
        }
        let runner = self
            .runner
            .as_mut()
            .ok_or_else(|| "llm runner not initialized".to_string())?;
        let mut agent_ids: Vec<String> = kernel.model().agents.keys().cloned().collect();
        agent_ids.sort();
        for agent_id in agent_ids {
            if runner.get(agent_id.as_str()).is_some() {
                continue;
            }
            let mut behavior = LlmAgentBehavior::from_env(agent_id.clone())
                .map_err(|err| format!("llm init failed for {}: {err}", agent_id))?;
            if let Some(profile) = self.prompt_profiles.get(agent_id.as_str()) {
                behavior.apply_prompt_overrides(
                    profile.system_prompt_override.clone(),
                    profile.short_term_goal_override.clone(),
                    profile.long_term_goal_override.clone(),
                );
            }
            restore_behavior_long_term_memory_from_model(&mut behavior, kernel, agent_id.as_str());
            runner.register(behavior);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct RuntimePendingAction {
    agent_id: String,
    action: SimulatorAction,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeLlmDecision {
    pub(super) agent_id: String,
    pub(super) decision: AgentDecision,
    pub(super) decision_trace: Option<AgentDecisionTrace>,
}

impl RuntimeLlmDecision {
    fn from_error(world: &RuntimeWorld, message: String) -> Self {
        let agent_id = world
            .state()
            .agents
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "runtime-agent-0".to_string());
        let trace = AgentDecisionTrace {
            agent_id: agent_id.clone(),
            time: world.state().time,
            decision: AgentDecision::Wait,
            llm_input: None,
            llm_output: None,
            llm_error: Some(message),
            parse_error: None,
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
            llm_chat_messages: Vec::new(),
        };
        Self {
            agent_id,
            decision: AgentDecision::Wait,
            decision_trace: Some(trace),
        }
    }
}

pub(super) fn normalize_required_player_id(
    player_id: &str,
    agent_id: &str,
) -> Result<String, PromptControlError> {
    let normalized = player_id.trim();
    if normalized.is_empty() {
        return Err(PromptControlError {
            code: "player_id_required".to_string(),
            message: format!(
                "prompt_control for {} requires non-empty player_id",
                agent_id
            ),
            agent_id: Some(agent_id.to_string()),
            current_version: None,
        });
    }
    Ok(normalized.to_string())
}

pub(super) fn normalize_optional_public_key(public_key: Option<&str>) -> Option<String> {
    public_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn ensure_updated_by_matches_player_runtime(
    updated_by: Option<&str>,
    player_id: &str,
    agent_id: &str,
) -> Result<(), PromptControlError> {
    let Some(updated_by) = updated_by.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if updated_by == player_id {
        return Ok(());
    }
    Err(PromptControlError {
        code: "updated_by_mismatch".to_string(),
        message: format!(
            "updated_by ({}) must match player_id ({}) for {}",
            updated_by, player_id, agent_id
        ),
        agent_id: Some(agent_id.to_string()),
        current_version: None,
    })
}

pub(super) fn ensure_agent_player_access_runtime(
    world: &RuntimeWorld,
    sidecar: &RuntimeLlmSidecar,
    agent_id: &str,
    player_id: &str,
    public_key: Option<&str>,
) -> Result<(), PromptControlError> {
    if !world.state().agents.contains_key(agent_id) {
        return Err(PromptControlError {
            code: "agent_not_found".to_string(),
            message: format!("agent not found: {agent_id}"),
            agent_id: Some(agent_id.to_string()),
            current_version: None,
        });
    }
    let Some(bound_player_id) = sidecar.agent_player_bindings.get(agent_id) else {
        return Ok(());
    };
    if bound_player_id == player_id {
        let Some(bound_public_key) = sidecar.agent_public_key_bindings.get(agent_id) else {
            return Ok(());
        };
        let requested_public_key = normalize_optional_public_key(public_key);
        if requested_public_key.as_deref() == Some(bound_public_key.as_str()) {
            return Ok(());
        }
        let message = if requested_public_key.is_none() {
            format!(
                "agent {} is bound to player {} with public_key {}, public_key is required",
                agent_id, bound_player_id, bound_public_key
            )
        } else {
            format!(
                "agent {} is bound to player {} with different public_key",
                agent_id, bound_player_id
            )
        };
        return Err(PromptControlError {
            code: "agent_control_forbidden".to_string(),
            message,
            agent_id: Some(agent_id.to_string()),
            current_version: sidecar
                .prompt_profiles
                .get(agent_id)
                .map(|entry| entry.version),
        });
    }
    Err(PromptControlError {
        code: "agent_control_forbidden".to_string(),
        message: format!(
            "agent {} is bound to player {}, not {}",
            agent_id, bound_player_id, player_id
        ),
        agent_id: Some(agent_id.to_string()),
        current_version: sidecar
            .prompt_profiles
            .get(agent_id)
            .map(|entry| entry.version),
    })
}

pub(super) fn apply_prompt_patch_runtime(
    profile: &mut AgentPromptProfile,
    request: &PromptControlApplyRequest,
) {
    if let Some(next) = &request.system_prompt_override {
        profile.system_prompt_override = sanitize_patch_string(next.clone());
    }
    if let Some(next) = &request.short_term_goal_override {
        profile.short_term_goal_override = sanitize_patch_string(next.clone());
    }
    if let Some(next) = &request.long_term_goal_override {
        profile.long_term_goal_override = sanitize_patch_string(next.clone());
    }
}

fn sanitize_patch_string(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

pub(super) fn changed_prompt_fields_runtime(
    current: &AgentPromptProfile,
    candidate: &AgentPromptProfile,
) -> Vec<String> {
    let mut fields = Vec::new();
    if current.system_prompt_override != candidate.system_prompt_override {
        fields.push("system_prompt_override".to_string());
    }
    if current.short_term_goal_override != candidate.short_term_goal_override {
        fields.push("short_term_goal_override".to_string());
    }
    if current.long_term_goal_override != candidate.long_term_goal_override {
        fields.push("long_term_goal_override".to_string());
    }
    fields
}

pub(super) fn prompt_profile_digest_runtime(profile: &AgentPromptProfile) -> String {
    let payload = serde_json::json!({
        "agent_id": profile.agent_id,
        "system_prompt_override": profile.system_prompt_override,
        "short_term_goal_override": profile.short_term_goal_override,
        "long_term_goal_override": profile.long_term_goal_override,
    });
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub(super) fn ensure_expected_prompt_version_runtime(
    agent_id: &str,
    current_version: u64,
    expected_version: Option<u64>,
) -> Result<(), PromptControlError> {
    if let Some(expected) = expected_version {
        if expected != current_version {
            return Err(PromptControlError {
                code: "version_conflict".to_string(),
                message: format!(
                    "prompt profile version conflict for {}: expected {}, current {}",
                    agent_id, expected, current_version
                ),
                agent_id: Some(agent_id.to_string()),
                current_version: Some(current_version),
            });
        }
    }
    Ok(())
}

pub(super) fn map_auth_verify_error_code(message: &str) -> &'static str {
    if message.contains("nonce") {
        return "auth_nonce_invalid";
    }
    if message.contains("signature") || message.contains("awviewauth:v1") {
        return "auth_signature_invalid";
    }
    if message.contains("player_id") || message.contains("public_key") {
        return "auth_claim_mismatch";
    }
    if message.contains("required") || message.contains("empty") {
        return "auth_claim_invalid";
    }
    "auth_invalid"
}

pub(super) fn simulator_action_label(action: &SimulatorAction) -> String {
    format!("{action:?}")
}

pub(super) fn simulator_action_to_runtime(
    action: &SimulatorAction,
    world: &RuntimeWorld,
) -> Option<RuntimeAction> {
    match action {
        SimulatorAction::RegisterAgent {
            agent_id,
            location_id,
        } => Some(RuntimeAction::RegisterAgent {
            agent_id: agent_id.clone(),
            pos: resolve_runtime_location(world, location_id)?,
        }),
        SimulatorAction::MoveAgent { agent_id, to } => Some(RuntimeAction::MoveAgent {
            agent_id: agent_id.clone(),
            to: resolve_runtime_location(world, to)?,
        }),
        SimulatorAction::TransferResource {
            from,
            to,
            kind,
            amount,
        } => match (from, to) {
            (
                ResourceOwner::Agent {
                    agent_id: from_agent_id,
                },
                ResourceOwner::Agent {
                    agent_id: to_agent_id,
                },
            ) => Some(RuntimeAction::TransferResource {
                from_agent_id: from_agent_id.clone(),
                to_agent_id: to_agent_id.clone(),
                kind: *kind,
                amount: *amount,
            }),
            _ => None,
        },
        SimulatorAction::FormAlliance {
            proposer_agent_id,
            alliance_id,
            members,
            charter,
        } => Some(RuntimeAction::FormAlliance {
            proposer_agent_id: proposer_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            members: members.clone(),
            charter: charter.clone(),
        }),
        SimulatorAction::JoinAlliance {
            operator_agent_id,
            alliance_id,
            member_agent_id,
        } => Some(RuntimeAction::JoinAlliance {
            operator_agent_id: operator_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            member_agent_id: member_agent_id.clone(),
        }),
        SimulatorAction::LeaveAlliance {
            operator_agent_id,
            alliance_id,
            member_agent_id,
        } => Some(RuntimeAction::LeaveAlliance {
            operator_agent_id: operator_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            member_agent_id: member_agent_id.clone(),
        }),
        SimulatorAction::DissolveAlliance {
            operator_agent_id,
            alliance_id,
            reason,
        } => Some(RuntimeAction::DissolveAlliance {
            operator_agent_id: operator_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            reason: reason.clone(),
        }),
        SimulatorAction::DeclareWar {
            initiator_agent_id,
            war_id,
            aggressor_alliance_id,
            defender_alliance_id,
            objective,
            intensity,
        } => Some(RuntimeAction::DeclareWar {
            initiator_agent_id: initiator_agent_id.clone(),
            war_id: war_id.clone(),
            aggressor_alliance_id: aggressor_alliance_id.clone(),
            defender_alliance_id: defender_alliance_id.clone(),
            objective: objective.clone(),
            intensity: *intensity,
        }),
        SimulatorAction::OpenGovernanceProposal {
            proposer_agent_id,
            proposal_key,
            title,
            description,
            options,
            voting_window_ticks,
            quorum_weight,
            pass_threshold_bps,
        } => Some(RuntimeAction::OpenGovernanceProposal {
            proposer_agent_id: proposer_agent_id.clone(),
            proposal_key: proposal_key.clone(),
            title: title.clone(),
            description: description.clone(),
            options: options.clone(),
            voting_window_ticks: *voting_window_ticks,
            quorum_weight: *quorum_weight,
            pass_threshold_bps: *pass_threshold_bps,
        }),
        SimulatorAction::CastGovernanceVote {
            voter_agent_id,
            proposal_key,
            option,
            weight,
        } => Some(RuntimeAction::CastGovernanceVote {
            voter_agent_id: voter_agent_id.clone(),
            proposal_key: proposal_key.clone(),
            option: option.clone(),
            weight: *weight,
        }),
        SimulatorAction::ResolveCrisis {
            resolver_agent_id,
            crisis_id,
            strategy,
            success,
        } => Some(RuntimeAction::ResolveCrisis {
            resolver_agent_id: resolver_agent_id.clone(),
            crisis_id: crisis_id.clone(),
            strategy: strategy.clone(),
            success: *success,
        }),
        SimulatorAction::GrantMetaProgress {
            operator_agent_id,
            target_agent_id,
            track,
            points,
            achievement_id,
        } => Some(RuntimeAction::GrantMetaProgress {
            operator_agent_id: operator_agent_id.clone(),
            target_agent_id: target_agent_id.clone(),
            track: track.clone(),
            points: *points,
            achievement_id: achievement_id.clone(),
        }),
        SimulatorAction::OpenEconomicContract {
            creator_agent_id,
            contract_id,
            counterparty_agent_id,
            settlement_kind,
            settlement_amount,
            reputation_stake,
            expires_at,
            description,
        } => Some(RuntimeAction::OpenEconomicContract {
            creator_agent_id: creator_agent_id.clone(),
            contract_id: contract_id.clone(),
            counterparty_agent_id: counterparty_agent_id.clone(),
            settlement_kind: *settlement_kind,
            settlement_amount: *settlement_amount,
            reputation_stake: *reputation_stake,
            expires_at: *expires_at,
            description: description.clone(),
        }),
        SimulatorAction::AcceptEconomicContract {
            accepter_agent_id,
            contract_id,
        } => Some(RuntimeAction::AcceptEconomicContract {
            accepter_agent_id: accepter_agent_id.clone(),
            contract_id: contract_id.clone(),
        }),
        SimulatorAction::SettleEconomicContract {
            operator_agent_id,
            contract_id,
            success,
            notes,
        } => Some(RuntimeAction::SettleEconomicContract {
            operator_agent_id: operator_agent_id.clone(),
            contract_id: contract_id.clone(),
            success: *success,
            notes: notes.clone(),
        }),
        SimulatorAction::CompileModuleArtifactFromSource {
            publisher_agent_id,
            module_id,
            manifest_path,
            source_files,
        } => Some(RuntimeAction::CompileModuleArtifactFromSource {
            publisher_agent_id: publisher_agent_id.clone(),
            module_id: module_id.clone(),
            source_package: ModuleSourcePackage {
                manifest_path: manifest_path.clone(),
                files: source_files.clone(),
            },
        }),
        SimulatorAction::DeployModuleArtifact {
            publisher_agent_id,
            wasm_hash,
            wasm_bytes,
            ..
        } => Some(RuntimeAction::DeployModuleArtifact {
            publisher_agent_id: publisher_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
            wasm_bytes: wasm_bytes.clone(),
        }),
        SimulatorAction::ListModuleArtifactForSale {
            seller_agent_id,
            wasm_hash,
            price_kind,
            price_amount,
        } => Some(RuntimeAction::ListModuleArtifactForSale {
            seller_agent_id: seller_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
            price_kind: *price_kind,
            price_amount: *price_amount,
        }),
        SimulatorAction::BuyModuleArtifact {
            buyer_agent_id,
            wasm_hash,
        } => Some(RuntimeAction::BuyModuleArtifact {
            buyer_agent_id: buyer_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
        }),
        SimulatorAction::DelistModuleArtifact {
            seller_agent_id,
            wasm_hash,
        } => Some(RuntimeAction::DelistModuleArtifact {
            seller_agent_id: seller_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
        }),
        SimulatorAction::DestroyModuleArtifact {
            owner_agent_id,
            wasm_hash,
            reason,
        } => Some(RuntimeAction::DestroyModuleArtifact {
            owner_agent_id: owner_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
            reason: reason.clone(),
        }),
        SimulatorAction::PlaceModuleArtifactBid {
            bidder_agent_id,
            wasm_hash,
            price_kind,
            price_amount,
        } => Some(RuntimeAction::PlaceModuleArtifactBid {
            bidder_agent_id: bidder_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
            price_kind: *price_kind,
            price_amount: *price_amount,
        }),
        SimulatorAction::CancelModuleArtifactBid {
            bidder_agent_id,
            wasm_hash,
            bid_order_id,
        } => Some(RuntimeAction::CancelModuleArtifactBid {
            bidder_agent_id: bidder_agent_id.clone(),
            wasm_hash: wasm_hash.clone(),
            bid_order_id: *bid_order_id,
        }),
        _ => None,
    }
}

fn resolve_runtime_location(world: &RuntimeWorld, location_id: &str) -> Option<GeoPos> {
    if let Some(pos) = parse_runtime_location_id(location_id) {
        return Some(pos);
    }
    world
        .state()
        .agents
        .values()
        .map(|cell| cell.state.pos)
        .find(|pos| location_id_for_pos(*pos) == location_id)
}

fn parse_runtime_location_id(location_id: &str) -> Option<GeoPos> {
    let raw = location_id.strip_prefix("runtime:")?;
    let mut parts = raw.split(':');
    let x = parts.next()?.parse::<i64>().ok()?;
    let y = parts.next()?.parse::<i64>().ok()?;
    let z = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(GeoPos::new(x as f64, y as f64, z as f64))
}

fn restore_behavior_long_term_memory_from_model(
    behavior: &mut LlmAgentBehavior<OpenAiChatCompletionClient>,
    kernel: &WorldKernel,
    agent_id: &str,
) {
    if let Some(entries) = kernel.long_term_memory_for_agent(agent_id) {
        behavior.restore_long_term_memory_entries(entries);
    } else {
        behavior.restore_long_term_memory_entries(&[]);
    }
}

fn sync_llm_runner_long_term_memory(
    kernel: &mut WorldKernel,
    runner: &AgentRunner<LlmAgentBehavior<OpenAiChatCompletionClient>>,
) {
    for agent_id in runner.agent_ids() {
        let Some(agent) = runner.get(agent_id.as_str()) else {
            continue;
        };
        let entries = agent.behavior.export_long_term_memory_entries();
        if let Err(message) = kernel.set_agent_long_term_memory(agent_id.as_str(), entries) {
            eprintln!(
                "viewer runtime live: skip long-term memory sync for {}: {}",
                agent_id, message
            );
        }
    }
}
