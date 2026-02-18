use super::super::{
    Action, ActionEnvelope, CausedBy, DomainEvent, ModuleActivation, ModuleChangeSet,
    ProposalDecision, RejectReason, WorldError, WorldEventBody,
};
use super::World;

impl World {
    pub(super) fn try_apply_runtime_module_action(
        &mut self,
        envelope: &ActionEnvelope,
    ) -> Result<bool, WorldError> {
        let action_id = envelope.id;
        match &envelope.action {
            Action::DeployModuleArtifact {
                publisher_agent_id,
                wasm_hash,
                wasm_bytes,
            } => {
                if !self.state.agents.contains_key(publisher_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: publisher_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                match self.register_module_artifact(wasm_hash.clone(), wasm_bytes.as_slice()) {
                    Ok(()) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ModuleArtifactDeployed {
                                publisher_agent_id: publisher_agent_id.clone(),
                                wasm_hash: wasm_hash.clone(),
                                bytes_len: wasm_bytes.len() as u64,
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                    }
                    Err(err) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ActionRejected {
                                action_id,
                                reason: RejectReason::RuleDenied {
                                    notes: vec![format!(
                                        "deploy module artifact rejected: {err:?}"
                                    )],
                                },
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                    }
                }

                Ok(true)
            }
            Action::InstallModuleFromArtifact {
                installer_agent_id,
                manifest,
                activate,
            } => {
                if !self.state.agents.contains_key(installer_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: installer_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let mut changes = ModuleChangeSet {
                    register: vec![manifest.clone()],
                    ..ModuleChangeSet::default()
                };
                if *activate {
                    changes.activate.push(ModuleActivation {
                        module_id: manifest.module_id.clone(),
                        version: manifest.version.clone(),
                    });
                }

                let module_changes_value = match serde_json::to_value(&changes) {
                    Ok(value) => value,
                    Err(err) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ActionRejected {
                                action_id,
                                reason: RejectReason::RuleDenied {
                                    notes: vec![format!("serialize module changes failed: {err}")],
                                },
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                        return Ok(true);
                    }
                };

                let mut manifest_update = self.manifest.clone();
                manifest_update.version = manifest_update.version.saturating_add(1);
                let serde_json::Value::Object(content) = &mut manifest_update.content else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec!["current manifest content must be object".to_string()],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                content.insert("module_changes".to_string(), module_changes_value);

                let proposal_id = match self
                    .propose_manifest_update(manifest_update, installer_agent_id.clone())
                {
                    Ok(proposal_id) => proposal_id,
                    Err(err) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ActionRejected {
                                action_id,
                                reason: RejectReason::RuleDenied {
                                    notes: vec![format!(
                                        "propose module install rejected: {err:?}"
                                    )],
                                },
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                        return Ok(true);
                    }
                };

                if let Err(err) = self.shadow_proposal(proposal_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!("shadow module install rejected: {err:?}")],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                if let Err(err) = self.approve_proposal(
                    proposal_id,
                    installer_agent_id.clone(),
                    ProposalDecision::Approve,
                ) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!("approve module install rejected: {err:?}")],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let manifest_hash = match self.apply_proposal(proposal_id) {
                    Ok(hash) => hash,
                    Err(err) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ActionRejected {
                                action_id,
                                reason: RejectReason::RuleDenied {
                                    notes: vec![format!("apply module install rejected: {err:?}")],
                                },
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                        return Ok(true);
                    }
                };

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleInstalled {
                        installer_agent_id: installer_agent_id.clone(),
                        module_id: manifest.module_id.clone(),
                        module_version: manifest.version.clone(),
                        active: *activate,
                        proposal_id,
                        manifest_hash,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
