use super::super::{
    Action, ActionEnvelope, CausedBy, DomainEvent, ModuleActivation, ModuleChangeSet,
    ProposalDecision, RejectReason, WorldError, WorldEventBody,
};
use super::World;
use crate::simulator::ResourceKind;

const MODULE_DEPLOY_FEE_BYTES_PER_ELECTRICITY: i64 = 2_048;
const MODULE_LIST_FEE_AMOUNT: i64 = 1;
const MODULE_DELIST_FEE_AMOUNT: i64 = 1;
const MODULE_DESTROY_FEE_AMOUNT: i64 = 1;

impl World {
    fn module_deploy_fee_amount(bytes_len: usize) -> i64 {
        let bytes_len = bytes_len as i64;
        (bytes_len.saturating_add(MODULE_DEPLOY_FEE_BYTES_PER_ELECTRICITY - 1)
            / MODULE_DEPLOY_FEE_BYTES_PER_ELECTRICITY)
            .max(1)
    }

    fn module_install_fee_amount(manifest: &agent_world_wasm_abi::ModuleManifest) -> i64 {
        let export_cost = manifest.exports.len() as i64;
        let subscription_cost = manifest.subscriptions.len() as i64;
        1_i64
            .saturating_add(export_cost)
            .saturating_add(subscription_cost)
            .max(1)
    }

    fn ensure_module_fee_affordable(
        &mut self,
        action_id: u64,
        agent_id: &str,
        fee_kind: ResourceKind,
        fee_amount: i64,
        action_name: &str,
    ) -> Result<bool, WorldError> {
        if fee_amount <= 0 {
            return Ok(true);
        }
        let available = self
            .state
            .agents
            .get(agent_id)
            .map(|cell| cell.state.resources.get(fee_kind))
            .unwrap_or(0);
        if available < fee_amount {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::InsufficientResource {
                        agent_id: agent_id.to_string(),
                        kind: fee_kind,
                        requested: fee_amount,
                        available,
                    },
                }),
                Some(CausedBy::Action(action_id)),
            )?;
            return Ok(false);
        }
        let _ = action_name;
        Ok(true)
    }

    fn has_active_module_using_artifact(&self, wasm_hash: &str) -> bool {
        self.module_registry
            .active
            .iter()
            .any(|(module_id, version)| {
                let key = agent_world_wasm_abi::ModuleRegistry::record_key(module_id, version);
                self.module_registry
                    .records
                    .get(&key)
                    .map(|record| record.manifest.wasm_hash == wasm_hash)
                    .unwrap_or(false)
            })
    }

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

                let computed_hash = super::super::util::sha256_hex(wasm_bytes);
                if computed_hash != *wasm_hash {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "deploy module artifact rejected: artifact hash mismatch expected {} found {}",
                                    wasm_hash, computed_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let fee_kind = ResourceKind::Electricity;
                let fee_amount = Self::module_deploy_fee_amount(wasm_bytes.len());
                if !self.ensure_module_fee_affordable(
                    action_id,
                    publisher_agent_id.as_str(),
                    fee_kind,
                    fee_amount,
                    "deploy_module_artifact",
                )? {
                    return Ok(true);
                }

                match self.register_module_artifact(wasm_hash.clone(), wasm_bytes.as_slice()) {
                    Ok(()) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ModuleArtifactDeployed {
                                publisher_agent_id: publisher_agent_id.clone(),
                                wasm_hash: wasm_hash.clone(),
                                bytes_len: wasm_bytes.len() as u64,
                                fee_kind,
                                fee_amount,
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
                if let Some(owner_agent_id) =
                    self.state.module_artifact_owners.get(&manifest.wasm_hash)
                {
                    if owner_agent_id != installer_agent_id {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ActionRejected {
                                action_id,
                                reason: RejectReason::RuleDenied {
                                    notes: vec![format!(
                                        "install module artifact rejected: installer {} does not own {} (owner {})",
                                        installer_agent_id, manifest.wasm_hash, owner_agent_id
                                    )],
                                },
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                        return Ok(true);
                    }
                }
                let fee_kind = ResourceKind::Electricity;
                let fee_amount = Self::module_install_fee_amount(manifest);
                if !self.ensure_module_fee_affordable(
                    action_id,
                    installer_agent_id.as_str(),
                    fee_kind,
                    fee_amount,
                    "install_module_from_artifact",
                )? {
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
                        fee_kind,
                        fee_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                Ok(true)
            }
            Action::ListModuleArtifactForSale {
                seller_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
            } => {
                if !self.state.agents.contains_key(seller_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: seller_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                if *price_amount <= 0 {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::InvalidAmount {
                                amount: *price_amount,
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                if !self.module_artifacts.contains(wasm_hash) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "list module artifact rejected: missing artifact {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                let Some(owner_agent_id) = self.state.module_artifact_owners.get(wasm_hash) else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "list module artifact rejected: owner missing for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if owner_agent_id != seller_agent_id {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "list module artifact rejected: seller {} does not own {} (owner {})",
                                    seller_agent_id, wasm_hash, owner_agent_id
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let fee_kind = ResourceKind::Data;
                let fee_amount = MODULE_LIST_FEE_AMOUNT;
                if !self.ensure_module_fee_affordable(
                    action_id,
                    seller_agent_id.as_str(),
                    fee_kind,
                    fee_amount,
                    "list_module_artifact_for_sale",
                )? {
                    return Ok(true);
                }

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactListed {
                        seller_agent_id: seller_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        price_kind: *price_kind,
                        price_amount: *price_amount,
                        fee_kind,
                        fee_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                Ok(true)
            }
            Action::BuyModuleArtifact {
                buyer_agent_id,
                wasm_hash,
            } => {
                if !self.state.agents.contains_key(buyer_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: buyer_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let Some(listing) = self.state.module_artifact_listings.get(wasm_hash).cloned()
                else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "buy module artifact rejected: listing missing for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if listing.seller_agent_id == *buyer_agent_id {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "buy module artifact rejected: buyer {} already owns listing {}",
                                    buyer_agent_id, wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                if !self.state.agents.contains_key(&listing.seller_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: listing.seller_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let available = self
                    .state
                    .agents
                    .get(buyer_agent_id)
                    .map(|cell| cell.state.resources.get(listing.price_kind))
                    .unwrap_or(0);
                if available < listing.price_amount {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::InsufficientResource {
                                agent_id: buyer_agent_id.clone(),
                                kind: listing.price_kind,
                                requested: listing.price_amount,
                                available,
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactSaleCompleted {
                        buyer_agent_id: buyer_agent_id.clone(),
                        seller_agent_id: listing.seller_agent_id,
                        wasm_hash: wasm_hash.clone(),
                        price_kind: listing.price_kind,
                        price_amount: listing.price_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                Ok(true)
            }
            Action::DelistModuleArtifact {
                seller_agent_id,
                wasm_hash,
            } => {
                if !self.state.agents.contains_key(seller_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: seller_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let Some(listing) = self.state.module_artifact_listings.get(wasm_hash) else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "delist module artifact rejected: listing missing for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if listing.seller_agent_id != *seller_agent_id {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "delist module artifact rejected: seller {} does not own listing {}",
                                    seller_agent_id, wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                let Some(owner_agent_id) = self.state.module_artifact_owners.get(wasm_hash) else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "delist module artifact rejected: owner missing for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if owner_agent_id != seller_agent_id {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "delist module artifact rejected: seller {} does not own {} (owner {})",
                                    seller_agent_id, wasm_hash, owner_agent_id
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let fee_kind = ResourceKind::Data;
                let fee_amount = MODULE_DELIST_FEE_AMOUNT;
                if !self.ensure_module_fee_affordable(
                    action_id,
                    seller_agent_id.as_str(),
                    fee_kind,
                    fee_amount,
                    "delist_module_artifact",
                )? {
                    return Ok(true);
                }

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactDelisted {
                        seller_agent_id: seller_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        fee_kind,
                        fee_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                Ok(true)
            }
            Action::DestroyModuleArtifact {
                owner_agent_id,
                wasm_hash,
                reason,
            } => {
                if !self.state.agents.contains_key(owner_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: owner_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                if reason.trim().is_empty() {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![
                                    "destroy module artifact rejected: reason is empty".to_string()
                                ],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                if !self.module_artifacts.contains(wasm_hash) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "destroy module artifact rejected: missing artifact {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let Some(owner) = self.state.module_artifact_owners.get(wasm_hash) else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "destroy module artifact rejected: owner missing for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if owner != owner_agent_id {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "destroy module artifact rejected: owner {} does not own {} (owner {})",
                                    owner_agent_id, wasm_hash, owner
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                if self.has_active_module_using_artifact(wasm_hash) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "destroy module artifact rejected: artifact {} is used by active module",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let fee_kind = ResourceKind::Electricity;
                let fee_amount = MODULE_DESTROY_FEE_AMOUNT;
                if !self.ensure_module_fee_affordable(
                    action_id,
                    owner_agent_id.as_str(),
                    fee_kind,
                    fee_amount,
                    "destroy_module_artifact",
                )? {
                    return Ok(true);
                }

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactDestroyed {
                        owner_agent_id: owner_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        reason: reason.clone(),
                        fee_kind,
                        fee_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                self.module_artifacts.remove(wasm_hash);
                self.module_artifact_bytes.remove(wasm_hash);
                let max_cached = self.module_cache.max_cached_modules();
                self.module_cache = agent_world_wasm_abi::ModuleCache::new(max_cached);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
