use super::super::state::{ModuleArtifactBidState, ModuleArtifactListingState};
use super::super::{
    Action, ActionEnvelope, CausedBy, DomainEvent, ModuleActivation, ModuleChangeSet,
    ProposalDecision, RejectReason, WorldError, WorldEventBody,
};
use super::World;
use crate::simulator::{ModuleInstallTarget, ResourceKind};

const MODULE_DEPLOY_FEE_BYTES_PER_ELECTRICITY: i64 = 2_048;
const MODULE_COMPILE_FEE_BYTES_PER_ELECTRICITY: i64 = 1_024;
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

    fn module_compile_fee_amount(source_bytes_len: usize, wasm_bytes_len: usize) -> i64 {
        let total_bytes = source_bytes_len.saturating_add(wasm_bytes_len) as i64;
        (total_bytes.saturating_add(MODULE_COMPILE_FEE_BYTES_PER_ELECTRICITY - 1)
            / MODULE_COMPILE_FEE_BYTES_PER_ELECTRICITY)
            .max(2)
    }

    fn module_install_fee_amount(manifest: &agent_world_wasm_abi::ModuleManifest) -> i64 {
        let export_cost = manifest.exports.len() as i64;
        let subscription_cost = manifest.subscriptions.len() as i64;
        1_i64
            .saturating_add(export_cost)
            .saturating_add(subscription_cost)
            .max(1)
    }

    fn next_module_instance_id(&self, module_id: &str) -> String {
        let seq = self.state.next_module_instance_id.max(1);
        format!("{module_id}#{seq}")
    }

    fn ensure_module_fee_affordable(
        &mut self,
        action_id: u64,
        agent_id: &str,
        fee_kind: ResourceKind,
        fee_amount: i64,
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
        Ok(true)
    }

    fn has_active_module_using_artifact(&self, wasm_hash: &str) -> bool {
        if self
            .state
            .module_instances
            .values()
            .any(|instance| instance.active && instance.wasm_hash == wasm_hash)
        {
            return true;
        }
        self.module_registry
            .active
            .iter()
            .any(|(module_id, version)| {
                if self
                    .state
                    .module_instances
                    .values()
                    .any(|instance| instance.module_id == *module_id)
                {
                    return false;
                }
                let key = agent_world_wasm_abi::ModuleRegistry::record_key(module_id, version);
                self.module_registry
                    .records
                    .get(&key)
                    .map(|record| record.manifest.wasm_hash == wasm_hash)
                    .unwrap_or(false)
            })
    }

    fn peek_next_module_market_order_id(&self) -> u64 {
        self.state.next_module_market_order_id.max(1)
    }

    fn peek_next_module_market_sale_id(&self) -> u64 {
        self.state.next_module_market_sale_id.max(1)
    }

    fn best_bid_for_listing(
        &self,
        wasm_hash: &str,
        listing: &ModuleArtifactListingState,
    ) -> Option<ModuleArtifactBidState> {
        let bids = self.state.module_artifact_bids.get(wasm_hash)?;
        let mut best: Option<ModuleArtifactBidState> = None;
        for bid in bids {
            if bid.price_kind != listing.price_kind {
                continue;
            }
            if bid.price_amount < listing.price_amount {
                continue;
            }
            if bid.bidder_agent_id == listing.seller_agent_id {
                continue;
            }
            let available = self
                .state
                .agents
                .get(&bid.bidder_agent_id)
                .map(|cell| cell.state.resources.get(listing.price_kind))
                .unwrap_or(0);
            if available < listing.price_amount {
                continue;
            }
            let replace = match &best {
                Some(current) => {
                    bid.price_amount > current.price_amount
                        || (bid.price_amount == current.price_amount
                            && bid.order_id < current.order_id)
                }
                None => true,
            };
            if replace {
                best = Some(bid.clone());
            }
        }
        best
    }

    fn try_match_module_listing(
        &mut self,
        wasm_hash: &str,
        action_id: u64,
    ) -> Result<(), WorldError> {
        let Some(listing) = self.state.module_artifact_listings.get(wasm_hash).cloned() else {
            return Ok(());
        };
        let Some(best_bid) = self.best_bid_for_listing(wasm_hash, &listing) else {
            return Ok(());
        };
        let sale_id = self.peek_next_module_market_sale_id();
        self.append_event(
            WorldEventBody::Domain(DomainEvent::ModuleArtifactSaleCompleted {
                buyer_agent_id: best_bid.bidder_agent_id,
                seller_agent_id: listing.seller_agent_id,
                wasm_hash: wasm_hash.to_string(),
                price_kind: listing.price_kind,
                price_amount: listing.price_amount,
                sale_id,
                listing_order_id: if listing.order_id > 0 {
                    Some(listing.order_id)
                } else {
                    None
                },
                bid_order_id: Some(best_bid.order_id),
            }),
            Some(CausedBy::Action(action_id)),
        )?;
        Ok(())
    }

    fn apply_install_module_action(
        &mut self,
        action_id: u64,
        installer_agent_id: &str,
        manifest: &agent_world_wasm_abi::ModuleManifest,
        activate: bool,
        install_target: ModuleInstallTarget,
    ) -> Result<bool, WorldError> {
        if !self.state.agents.contains_key(installer_agent_id) {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::AgentNotFound {
                        agent_id: installer_agent_id.to_string(),
                    },
                }),
                Some(CausedBy::Action(action_id)),
            )?;
            return Ok(true);
        }
        if let Some(owner_agent_id) = self.state.module_artifact_owners.get(&manifest.wasm_hash) {
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
            installer_agent_id,
            fee_kind,
            fee_amount,
        )? {
            return Ok(true);
        }

        let mut changes = ModuleChangeSet::default();
        let record_key = agent_world_wasm_abi::ModuleRegistry::record_key(
            manifest.module_id.as_str(),
            manifest.version.as_str(),
        );
        if let Some(record) = self.module_registry.records.get(record_key.as_str()) {
            if record.manifest != *manifest {
                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "install module rejected: existing manifest mismatch for {}",
                                record_key
                            )],
                        },
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                return Ok(true);
            }
        } else {
            changes.register.push(manifest.clone());
        }
        if activate {
            let already_active_same = self
                .module_registry
                .active
                .get(&manifest.module_id)
                .map(|version| version == &manifest.version)
                .unwrap_or(false);
            if !already_active_same {
                changes.activate.push(ModuleActivation {
                    module_id: manifest.module_id.clone(),
                    version: manifest.version.clone(),
                });
            }
        }

        let (proposal_id, manifest_hash) = if changes.is_empty() {
            (0, self.current_manifest_hash()?)
        } else {
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
                .propose_manifest_update(manifest_update, installer_agent_id.to_string())
            {
                Ok(proposal_id) => proposal_id,
                Err(err) => {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!("propose module install rejected: {err:?}")],
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
                installer_agent_id.to_string(),
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
            (proposal_id, manifest_hash)
        };

        let instance_id = self.next_module_instance_id(manifest.module_id.as_str());

        self.append_event(
            WorldEventBody::Domain(DomainEvent::ModuleInstalled {
                installer_agent_id: installer_agent_id.to_string(),
                instance_id,
                module_id: manifest.module_id.clone(),
                module_version: manifest.version.clone(),
                wasm_hash: manifest.wasm_hash.clone(),
                install_target,
                active: activate,
                proposal_id,
                manifest_hash,
                fee_kind,
                fee_amount,
            }),
            Some(CausedBy::Action(action_id)),
        )?;
        Ok(true)
    }

    pub(super) fn try_apply_runtime_module_action(
        &mut self,
        envelope: &ActionEnvelope,
    ) -> Result<bool, WorldError> {
        let action_id = envelope.id;
        match &envelope.action {
            Action::CompileModuleArtifactFromSource {
                publisher_agent_id,
                module_id,
                source_package,
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
                if module_id.trim().is_empty() {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec!["compile module source rejected: module_id is empty"
                                    .to_string()],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let source_bytes_len = source_package.files.values().map(Vec::len).sum::<usize>();
                let compiled_bytes =
                    match super::super::module_source_compiler::compile_module_artifact_from_source(
                        module_id.as_str(),
                        source_package,
                    ) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            self.append_event(
                                WorldEventBody::Domain(DomainEvent::ActionRejected {
                                    action_id,
                                    reason: RejectReason::RuleDenied {
                                        notes: vec![format!(
                                            "compile module source rejected: {err:?}"
                                        )],
                                    },
                                }),
                                Some(CausedBy::Action(action_id)),
                            )?;
                            return Ok(true);
                        }
                    };

                let wasm_hash = super::super::util::sha256_hex(&compiled_bytes);
                let fee_kind = ResourceKind::Electricity;
                let fee_amount =
                    Self::module_compile_fee_amount(source_bytes_len, compiled_bytes.len());
                if !self.ensure_module_fee_affordable(
                    action_id,
                    publisher_agent_id.as_str(),
                    fee_kind,
                    fee_amount,
                )? {
                    return Ok(true);
                }

                match self.register_module_artifact(wasm_hash.clone(), compiled_bytes.as_slice()) {
                    Ok(()) => {
                        self.append_event(
                            WorldEventBody::Domain(DomainEvent::ModuleArtifactDeployed {
                                publisher_agent_id: publisher_agent_id.clone(),
                                wasm_hash,
                                bytes_len: compiled_bytes.len() as u64,
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
                                        "compile module source rejected: register artifact failed: {err:?}"
                                    )],
                                },
                            }),
                            Some(CausedBy::Action(action_id)),
                        )?;
                    }
                }
                Ok(true)
            }
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
            } => self.apply_install_module_action(
                action_id,
                installer_agent_id.as_str(),
                manifest,
                *activate,
                ModuleInstallTarget::SelfAgent,
            ),
            Action::InstallModuleToTargetFromArtifact {
                installer_agent_id,
                manifest,
                activate,
                install_target,
            } => self.apply_install_module_action(
                action_id,
                installer_agent_id.as_str(),
                manifest,
                *activate,
                install_target.clone(),
            ),
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
                )? {
                    return Ok(true);
                }
                let order_id = self.peek_next_module_market_order_id();

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactListed {
                        seller_agent_id: seller_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        price_kind: *price_kind,
                        price_amount: *price_amount,
                        order_id,
                        fee_kind,
                        fee_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                self.try_match_module_listing(wasm_hash.as_str(), action_id)?;
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
                let sale_id = self.peek_next_module_market_sale_id();

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactSaleCompleted {
                        buyer_agent_id: buyer_agent_id.clone(),
                        seller_agent_id: listing.seller_agent_id,
                        wasm_hash: wasm_hash.clone(),
                        price_kind: listing.price_kind,
                        price_amount: listing.price_amount,
                        sale_id,
                        listing_order_id: if listing.order_id > 0 {
                            Some(listing.order_id)
                        } else {
                            None
                        },
                        bid_order_id: None,
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

                let Some(listing) = self.state.module_artifact_listings.get(wasm_hash).cloned()
                else {
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
                )? {
                    return Ok(true);
                }

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactDelisted {
                        seller_agent_id: seller_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        order_id: if listing.order_id > 0 {
                            Some(listing.order_id)
                        } else {
                            None
                        },
                        fee_kind,
                        fee_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                Ok(true)
            }
            Action::PlaceModuleArtifactBid {
                bidder_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
            } => {
                if !self.state.agents.contains_key(bidder_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: bidder_agent_id.clone(),
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
                                    "place module artifact bid rejected: missing artifact {}",
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
                                    "place module artifact bid rejected: owner missing for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if owner == bidder_agent_id {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "place module artifact bid rejected: bidder {} already owns {}",
                                    bidder_agent_id, wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                let available = self
                    .state
                    .agents
                    .get(bidder_agent_id)
                    .map(|cell| cell.state.resources.get(*price_kind))
                    .unwrap_or(0);
                if available < *price_amount {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::InsufficientResource {
                                agent_id: bidder_agent_id.clone(),
                                kind: *price_kind,
                                requested: *price_amount,
                                available,
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                let order_id = self.peek_next_module_market_order_id();
                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactBidPlaced {
                        bidder_agent_id: bidder_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        order_id,
                        price_kind: *price_kind,
                        price_amount: *price_amount,
                    }),
                    Some(CausedBy::Action(action_id)),
                )?;
                self.try_match_module_listing(wasm_hash.as_str(), action_id)?;
                Ok(true)
            }
            Action::CancelModuleArtifactBid {
                bidder_agent_id,
                wasm_hash,
                bid_order_id,
            } => {
                if !self.state.agents.contains_key(bidder_agent_id) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::AgentNotFound {
                                agent_id: bidder_agent_id.clone(),
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }
                let Some(bids) = self.state.module_artifact_bids.get(wasm_hash) else {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "cancel module artifact bid rejected: no bids for {}",
                                    wasm_hash
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                };
                if !bids.iter().any(|entry| {
                    entry.order_id == *bid_order_id && entry.bidder_agent_id == *bidder_agent_id
                }) {
                    self.append_event(
                        WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "cancel module artifact bid rejected: bid {} not found for {}",
                                    bid_order_id, bidder_agent_id
                                )],
                            },
                        }),
                        Some(CausedBy::Action(action_id)),
                    )?;
                    return Ok(true);
                }

                self.append_event(
                    WorldEventBody::Domain(DomainEvent::ModuleArtifactBidCancelled {
                        bidder_agent_id: bidder_agent_id.clone(),
                        wasm_hash: wasm_hash.clone(),
                        order_id: *bid_order_id,
                        reason: "cancelled_by_bidder".to_string(),
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
