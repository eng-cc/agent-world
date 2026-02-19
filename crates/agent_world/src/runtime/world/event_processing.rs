use super::super::{
    util::hash_json, Action, ActionEnvelope, ActionId, CausedBy, DomainEvent,
    EpochSettlementReport, MaterialLedgerId, MaterialStack, NodeRewardMintRecord, RejectReason,
    WorldError, WorldEvent, WorldEventBody, WorldEventId, WorldTime,
};
use super::body::{evaluate_expand_body_interface, validate_body_kernel_view};
use super::logistics::{
    MATERIAL_TRANSFER_LOSS_PER_KM_BPS, MATERIAL_TRANSFER_MAX_DISTANCE_KM,
    MATERIAL_TRANSFER_MAX_INFLIGHT, MATERIAL_TRANSFER_SPEED_KM_PER_TICK,
};
use super::World;
use crate::geometry::space_distance_cm;
use crate::simulator::ResourceKind;

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
            Action::QueryObservation { agent_id } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["observation requires rule module".to_string()],
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        },
                    }))
                }
            }
            Action::EmitObservation { observation } => {
                Ok(WorldEventBody::Domain(DomainEvent::Observation {
                    observation: observation.clone(),
                }))
            }
            Action::BodyAction { agent_id, .. } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["body action requires body module".to_string()],
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        },
                    }))
                }
            }
            Action::EmitBodyAttributes {
                agent_id,
                view,
                reason,
            } => {
                let Some(cell) = self.state.agents.get(agent_id) else {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        },
                    }));
                };
                if let Err(reason) = validate_body_kernel_view(&cell.state.body_view, view) {
                    return Ok(WorldEventBody::Domain(
                        DomainEvent::BodyAttributesRejected {
                            agent_id: agent_id.clone(),
                            reason,
                        },
                    ));
                }
                Ok(WorldEventBody::Domain(DomainEvent::BodyAttributesUpdated {
                    agent_id: agent_id.clone(),
                    view: view.clone(),
                    reason: reason.clone(),
                }))
            }
            Action::ExpandBodyInterface {
                agent_id,
                interface_module_item_id,
            } => Ok(evaluate_expand_body_interface(
                self,
                action_id,
                agent_id,
                interface_module_item_id,
            )),
            Action::DeployModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "deploy_module_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::CompileModuleArtifactFromSource { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "compile_module_artifact_from_source requires runtime action loop"
                                .to_string(),
                        ],
                    },
                }))
            }
            Action::InstallModuleFromArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "install_module_from_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::InstallModuleToTargetFromArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "install_module_to_target_from_artifact requires runtime action loop"
                                .to_string(),
                        ],
                    },
                }))
            }
            Action::UpgradeModuleFromArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "upgrade_module_from_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::ListModuleArtifactForSale { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["list_module_artifact_for_sale requires runtime action loop"
                            .to_string()],
                    },
                }))
            }
            Action::BuyModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["buy_module_artifact requires runtime action loop".to_string()],
                    },
                }))
            }
            Action::DelistModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "delist_module_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::DestroyModuleArtifact { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "destroy_module_artifact requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::PlaceModuleArtifactBid { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "place_module_artifact_bid requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::CancelModuleArtifactBid { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "cancel_module_artifact_bid requires runtime action loop".to_string()
                        ],
                    },
                }))
            }
            Action::TransferResource {
                from_agent_id,
                to_agent_id,
                ..
            } => {
                if !self.state.agents.contains_key(from_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: from_agent_id.clone(),
                        },
                    }));
                }
                if !self.state.agents.contains_key(to_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: to_agent_id.clone(),
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["transfer requires rule module".to_string()],
                    },
                }))
            }
            Action::RedeemPower {
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
            } => Ok(WorldEventBody::Domain(self.evaluate_redeem_power_action(
                node_id.as_str(),
                target_agent_id.as_str(),
                *redeem_credits,
                *nonce,
                None,
            ))),
            Action::RedeemPowerSigned {
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                signer_node_id,
                signature,
            } => Ok(WorldEventBody::Domain(self.evaluate_redeem_power_action(
                node_id.as_str(),
                target_agent_id.as_str(),
                *redeem_credits,
                *nonce,
                Some((signer_node_id.as_str(), signature.as_str())),
            ))),
            Action::ApplyNodePointsSettlementSigned {
                report,
                signer_node_id,
                mint_records,
            } => Ok(WorldEventBody::Domain(
                self.evaluate_apply_node_points_settlement_action(
                    action_id,
                    report,
                    signer_node_id.as_str(),
                    mint_records.as_slice(),
                ),
            )),
            Action::TransferMaterial {
                requester_agent_id,
                from_ledger,
                to_ledger,
                kind,
                amount,
                distance_km,
            } => {
                if !self.state.agents.contains_key(requester_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: requester_agent_id.clone(),
                        },
                    }));
                }
                if from_ledger == to_ledger {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["from_ledger and to_ledger cannot be the same".to_string()],
                        },
                    }));
                }
                if kind.trim().is_empty() {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["material kind cannot be empty".to_string()],
                        },
                    }));
                }
                if *amount <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InvalidAmount { amount: *amount },
                    }));
                }
                if *distance_km < 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["distance_km must be >= 0".to_string()],
                        },
                    }));
                }
                if *distance_km > MATERIAL_TRANSFER_MAX_DISTANCE_KM {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::MaterialTransferDistanceExceeded {
                            distance_km: *distance_km,
                            max_distance_km: MATERIAL_TRANSFER_MAX_DISTANCE_KM,
                        },
                    }));
                }
                let available = self.ledger_material_balance(from_ledger, kind.as_str());
                if available < *amount {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InsufficientMaterial {
                            material_kind: kind.clone(),
                            requested: *amount,
                            available,
                        },
                    }));
                }

                if *distance_km == 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::MaterialTransferred {
                        requester_agent_id: requester_agent_id.clone(),
                        from_ledger: from_ledger.clone(),
                        to_ledger: to_ledger.clone(),
                        kind: kind.clone(),
                        amount: *amount,
                        distance_km: *distance_km,
                    }));
                }

                if self.state.pending_material_transits.len() >= MATERIAL_TRANSFER_MAX_INFLIGHT {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::MaterialTransitCapacityExceeded {
                            in_flight: self.state.pending_material_transits.len(),
                            max_in_flight: MATERIAL_TRANSFER_MAX_INFLIGHT,
                        },
                    }));
                }

                let transit_ticks = ((*distance_km + MATERIAL_TRANSFER_SPEED_KM_PER_TICK - 1)
                    / MATERIAL_TRANSFER_SPEED_KM_PER_TICK)
                    .max(1) as u64;
                let ready_at = self.state.time.saturating_add(transit_ticks);
                Ok(WorldEventBody::Domain(
                    DomainEvent::MaterialTransitStarted {
                        job_id: action_id,
                        requester_agent_id: requester_agent_id.clone(),
                        from_ledger: from_ledger.clone(),
                        to_ledger: to_ledger.clone(),
                        kind: kind.clone(),
                        amount: *amount,
                        distance_km: *distance_km,
                        loss_bps: MATERIAL_TRANSFER_LOSS_PER_KM_BPS,
                        ready_at,
                    },
                ))
            }
            Action::EmitResourceTransfer {
                from_agent_id,
                to_agent_id,
                kind,
                amount,
            } => {
                if *amount <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InvalidAmount { amount: *amount },
                    }));
                }
                let Some(from_cell) = self.state.agents.get(from_agent_id) else {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: from_agent_id.clone(),
                        },
                    }));
                };
                let Some(to_cell) = self.state.agents.get(to_agent_id) else {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: to_agent_id.clone(),
                        },
                    }));
                };
                let distance_cm = space_distance_cm(from_cell.state.pos, to_cell.state.pos);
                if distance_cm > 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentsNotCoLocated {
                            agent_id: from_agent_id.clone(),
                            other_agent_id: to_agent_id.clone(),
                        },
                    }));
                }
                let available = from_cell.state.resources.get(*kind);
                if available < *amount {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InsufficientResource {
                            agent_id: from_agent_id.clone(),
                            kind: *kind,
                            requested: *amount,
                            available,
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::ResourceTransferred {
                    from_agent_id: from_agent_id.clone(),
                    to_agent_id: to_agent_id.clone(),
                    kind: *kind,
                    amount: *amount,
                }))
            }
            Action::BuildFactory {
                builder_agent_id,
                site_id,
                spec,
            } => {
                if !self.state.agents.contains_key(builder_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: builder_agent_id.clone(),
                        },
                    }));
                }
                if spec.factory_id.trim().is_empty() {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["factory_id cannot be empty".to_string()],
                        },
                    }));
                }
                if self.state.factories.contains_key(&spec.factory_id)
                    || self
                        .state
                        .pending_factory_builds
                        .values()
                        .any(|job| job.spec.factory_id == spec.factory_id)
                {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!("factory already exists: {}", spec.factory_id)],
                        },
                    }));
                }
                if spec.recipe_slots == 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["recipe_slots must be > 0".to_string()],
                        },
                    }));
                }
                let preferred_consume_ledger = MaterialLedgerId::agent(builder_agent_id.clone());
                let consume_ledger = self.select_material_consume_ledger_with_world_fallback(
                    preferred_consume_ledger,
                    &spec.build_cost,
                );
                for stack in &spec.build_cost {
                    if stack.amount <= 0 {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "factory build_cost must be > 0: {}={}",
                                    stack.kind, stack.amount
                                )],
                            },
                        }));
                    }
                    let available =
                        self.ledger_material_balance(&consume_ledger, stack.kind.as_str());
                    if available < stack.amount {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::InsufficientMaterial {
                                material_kind: stack.kind.clone(),
                                requested: stack.amount,
                                available,
                            },
                        }));
                    }
                }

                let build_ticks = spec.build_time_ticks.max(1);
                let ready_at = self.state.time.saturating_add(build_ticks as u64);
                Ok(WorldEventBody::Domain(DomainEvent::FactoryBuildStarted {
                    job_id: action_id,
                    builder_agent_id: builder_agent_id.clone(),
                    site_id: site_id.clone(),
                    spec: spec.clone(),
                    consume_ledger,
                    ready_at,
                }))
            }
            Action::BuildFactoryWithModule { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec!["build_factory_with_module requires module runtime".to_string()],
                    },
                }))
            }
            Action::ScheduleRecipe {
                requester_agent_id,
                factory_id,
                recipe_id,
                plan,
            } => {
                if !self.state.agents.contains_key(requester_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: requester_agent_id.clone(),
                        },
                    }));
                }
                let Some(factory) = self.state.factories.get(factory_id) else {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::FactoryNotFound {
                            factory_id: factory_id.clone(),
                        },
                    }));
                };
                let active_jobs = self
                    .state
                    .pending_recipe_jobs
                    .values()
                    .filter(|job| job.factory_id == *factory_id)
                    .count();
                if active_jobs >= factory.spec.recipe_slots as usize {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::FactoryBusy {
                            factory_id: factory_id.clone(),
                            active_jobs,
                            recipe_slots: factory.spec.recipe_slots,
                        },
                    }));
                }
                if let Some(reason) = &plan.reject_reason {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!("recipe plan rejected: {reason}")],
                        },
                    }));
                }
                if plan.accepted_batches == 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["recipe accepted_batches must be > 0".to_string()],
                        },
                    }));
                }
                let preferred_consume_ledger = factory.input_ledger.clone();
                let consume_ledger = self.select_material_consume_ledger_with_world_fallback(
                    preferred_consume_ledger,
                    &plan.consume,
                );
                let output_ledger = if consume_ledger == MaterialLedgerId::world() {
                    MaterialLedgerId::world()
                } else {
                    factory.output_ledger.clone()
                };
                for stack in &plan.consume {
                    if stack.amount <= 0 {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "recipe consume must be > 0: {}={}",
                                    stack.kind, stack.amount
                                )],
                            },
                        }));
                    }
                    let available =
                        self.ledger_material_balance(&consume_ledger, stack.kind.as_str());
                    if available < stack.amount {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::InsufficientMaterial {
                                material_kind: stack.kind.clone(),
                                requested: stack.amount,
                                available,
                            },
                        }));
                    }
                }
                if plan.power_required < 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["recipe power_required must be >= 0".to_string()],
                        },
                    }));
                }
                let available_power = self.resource_balance(ResourceKind::Electricity);
                if available_power < plan.power_required {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InsufficientResource {
                            agent_id: "world".to_string(),
                            kind: ResourceKind::Electricity,
                            requested: plan.power_required,
                            available: available_power,
                        },
                    }));
                }
                let duration_ticks = plan.duration_ticks.max(1);
                let ready_at = self.state.time.saturating_add(duration_ticks as u64);
                Ok(WorldEventBody::Domain(DomainEvent::RecipeStarted {
                    job_id: action_id,
                    requester_agent_id: requester_agent_id.clone(),
                    factory_id: factory_id.clone(),
                    recipe_id: recipe_id.clone(),
                    accepted_batches: plan.accepted_batches,
                    consume: plan.consume.clone(),
                    produce: plan.produce.clone(),
                    byproducts: plan.byproducts.clone(),
                    power_required: plan.power_required,
                    duration_ticks,
                    consume_ledger,
                    output_ledger,
                    ready_at,
                }))
            }
            Action::ScheduleRecipeWithModule { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "schedule_recipe_with_module requires module runtime".to_string()
                        ],
                    },
                }))
            }
            Action::ValidateProduct {
                requester_agent_id,
                module_id,
                stack,
                decision,
            } => {
                if !self.state.agents.contains_key(requester_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: requester_agent_id.clone(),
                        },
                    }));
                }
                if module_id.trim().is_empty() {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["product module_id cannot be empty".to_string()],
                        },
                    }));
                }
                if !decision.accepted {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: if decision.notes.is_empty() {
                                vec!["product validation rejected".to_string()]
                            } else {
                                decision.notes.clone()
                            },
                        },
                    }));
                }
                if decision.product_id != stack.kind {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "validated product mismatch expected={} got={}",
                                stack.kind, decision.product_id
                            )],
                        },
                    }));
                }
                if stack.amount <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["product stack amount must be > 0".to_string()],
                        },
                    }));
                }
                if stack.amount > decision.stack_limit as i64 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "product stack exceeds limit amount={} limit={}",
                                stack.amount, decision.stack_limit
                            )],
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::ProductValidated {
                    requester_agent_id: requester_agent_id.clone(),
                    module_id: module_id.clone(),
                    stack: stack.clone(),
                    stack_limit: decision.stack_limit,
                    tradable: decision.tradable,
                    quality_levels: decision.quality_levels.clone(),
                    notes: decision.notes.clone(),
                }))
            }
            Action::ValidateProductWithModule { .. } => {
                Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![
                            "validate_product_with_module requires module runtime".to_string()
                        ],
                    },
                }))
            }
        }
    }

    fn select_material_consume_ledger_with_world_fallback(
        &self,
        preferred_ledger: MaterialLedgerId,
        consume: &[MaterialStack],
    ) -> MaterialLedgerId {
        if self.has_materials_in_ledger(&preferred_ledger, consume) {
            preferred_ledger
        } else {
            MaterialLedgerId::world()
        }
    }

    fn evaluate_apply_node_points_settlement_action(
        &self,
        action_id: ActionId,
        report: &EpochSettlementReport,
        signer_node_id: &str,
        mint_records: &[NodeRewardMintRecord],
    ) -> DomainEvent {
        let settlement_hash = match hash_json(report) {
            Ok(hash) => hash,
            Err(err) => {
                return DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::RuleDenied {
                        notes: vec![format!("settlement hash compute failed: {err:?}")],
                    },
                };
            }
        };

        let event = DomainEvent::NodePointsSettlementApplied {
            report: report.clone(),
            signer_node_id: signer_node_id.to_string(),
            settlement_hash,
            minted_records: mint_records.to_vec(),
        };
        let mut preview_state = self.state.clone();
        if let Err(err) = preview_state.apply_domain_event(&event, self.state.time) {
            return DomainEvent::ActionRejected {
                action_id,
                reason: RejectReason::RuleDenied {
                    notes: vec![format!("apply node points settlement rejected: {err:?}")],
                },
            };
        }
        event
    }

    fn evaluate_redeem_power_action(
        &self,
        node_id: &str,
        target_agent_id: &str,
        redeem_credits: u64,
        nonce: u64,
        signed: Option<(&str, &str)>,
    ) -> DomainEvent {
        if self
            .state
            .reward_signature_governance_policy
            .require_redeem_signature
            && signed.is_none()
        {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "redeem signature is required by governance policy".to_string(),
            );
        }
        if let Some((signer_node_id, signature)) = signed {
            if signer_node_id.trim().is_empty() {
                return self.power_redeem_rejected(
                    node_id,
                    target_agent_id,
                    redeem_credits,
                    nonce,
                    "signer_node_id cannot be empty".to_string(),
                );
            }
            if signature.trim().is_empty() {
                return self.power_redeem_rejected(
                    node_id,
                    target_agent_id,
                    redeem_credits,
                    nonce,
                    "redeem signature cannot be empty".to_string(),
                );
            }
            if self
                .state
                .reward_signature_governance_policy
                .require_redeem_signer_match_node_id
                && signer_node_id != node_id
            {
                return self.power_redeem_rejected(
                    node_id,
                    target_agent_id,
                    redeem_credits,
                    nonce,
                    format!(
                        "redeem signer_node_id must match node_id by governance policy: signer={} node={}",
                        signer_node_id, node_id
                    ),
                );
            }
            if let Err(reason) = self.verify_redeem_power_signature(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                signer_node_id,
                signature,
            ) {
                return self.power_redeem_rejected(
                    node_id,
                    target_agent_id,
                    redeem_credits,
                    nonce,
                    format!("redeem signature verification failed: {reason}"),
                );
            }
        }

        if node_id.trim().is_empty() {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "node_id cannot be empty".to_string(),
            );
        }
        if !self.state.node_identity_bindings.contains_key(node_id) {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!("node identity not bound: {node_id}"),
            );
        }
        if !self.state.agents.contains_key(target_agent_id) {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!("target agent not found: {target_agent_id}"),
            );
        }
        if redeem_credits == 0 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "redeem_credits must be > 0".to_string(),
            );
        }
        if nonce == 0 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "nonce must be > 0".to_string(),
            );
        }
        if let Some(last_nonce) = self.state.node_redeem_nonces.get(node_id) {
            if nonce <= *last_nonce {
                return self.power_redeem_rejected(
                    node_id,
                    target_agent_id,
                    redeem_credits,
                    nonce,
                    format!(
                        "nonce replay detected: nonce={} last_nonce={}",
                        nonce, last_nonce
                    ),
                );
            }
        }
        let credits_per_power_unit = self.state.reward_asset_config.credits_per_power_unit;
        if credits_per_power_unit == 0 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "credits_per_power_unit must be positive".to_string(),
            );
        }
        let granted_power_units_u64 = redeem_credits / credits_per_power_unit;
        if granted_power_units_u64 == 0 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!(
                    "redeem credits below minimum conversion: credits={} per_unit={}",
                    redeem_credits, credits_per_power_unit
                ),
            );
        }
        if granted_power_units_u64 > i64::MAX as u64 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "granted power units overflow".to_string(),
            );
        }
        let granted_power_units = granted_power_units_u64 as i64;
        let min_redeem_power_unit = self.state.reward_asset_config.min_redeem_power_unit;
        if min_redeem_power_unit <= 0 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "min_redeem_power_unit must be positive".to_string(),
            );
        }
        if granted_power_units < min_redeem_power_unit {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!(
                    "granted power below minimum unit: granted={} min={}",
                    granted_power_units, min_redeem_power_unit
                ),
            );
        }
        let max_redeem_power_per_epoch = self.state.reward_asset_config.max_redeem_power_per_epoch;
        if max_redeem_power_per_epoch <= 0 {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                "max_redeem_power_per_epoch must be positive".to_string(),
            );
        }
        let next_redeemed = match self
            .state
            .protocol_power_reserve
            .redeemed_power_units
            .checked_add(granted_power_units)
        {
            Some(value) => value,
            None => {
                return self.power_redeem_rejected(
                    node_id,
                    target_agent_id,
                    redeem_credits,
                    nonce,
                    "redeemed_power_units overflow".to_string(),
                );
            }
        };
        if next_redeemed > max_redeem_power_per_epoch {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!(
                    "epoch redeem cap exceeded: next={} cap={}",
                    next_redeemed, max_redeem_power_per_epoch
                ),
            );
        }
        let available_credits = self.node_power_credit_balance(node_id);
        if available_credits < redeem_credits {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!(
                    "insufficient power credits: balance={} requested={}",
                    available_credits, redeem_credits
                ),
            );
        }
        if self.state.protocol_power_reserve.available_power_units < granted_power_units {
            return self.power_redeem_rejected(
                node_id,
                target_agent_id,
                redeem_credits,
                nonce,
                format!(
                    "insufficient protocol power reserve: available={} requested={}",
                    self.state.protocol_power_reserve.available_power_units, granted_power_units
                ),
            );
        }

        DomainEvent::PowerRedeemed {
            node_id: node_id.to_string(),
            target_agent_id: target_agent_id.to_string(),
            burned_credits: redeem_credits,
            granted_power_units,
            reserve_remaining: self.state.protocol_power_reserve.available_power_units
                - granted_power_units,
            nonce,
        }
    }

    fn power_redeem_rejected(
        &self,
        node_id: &str,
        target_agent_id: &str,
        redeem_credits: u64,
        nonce: u64,
        reason: String,
    ) -> DomainEvent {
        DomainEvent::PowerRedeemRejected {
            node_id: node_id.to_string(),
            target_agent_id: target_agent_id.to_string(),
            redeem_credits,
            nonce,
            reason,
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

    fn apply_event_body(
        &mut self,
        body: &WorldEventBody,
        time: WorldTime,
    ) -> Result<(), WorldError> {
        match body {
            WorldEventBody::Domain(event) => {
                self.state.apply_domain_event(event, time)?;
                self.state.route_domain_event(event);
                if let super::super::DomainEvent::ModuleInstalled {
                    instance_id,
                    module_id,
                    module_version,
                    active,
                    ..
                } = event
                {
                    let schedule_key = if instance_id.trim().is_empty() {
                        module_id.as_str()
                    } else {
                        instance_id.as_str()
                    };
                    if *active {
                        self.sync_tick_schedule_for_instance(
                            schedule_key,
                            module_id.as_str(),
                            module_version.as_str(),
                            time,
                        )?;
                    } else {
                        self.remove_tick_schedule(schedule_key);
                    }
                }
                if let super::super::DomainEvent::ModuleUpgraded {
                    instance_id,
                    module_id,
                    to_module_version,
                    active,
                    ..
                } = event
                {
                    if *active {
                        self.sync_tick_schedule_for_instance(
                            instance_id.as_str(),
                            module_id.as_str(),
                            to_module_version.as_str(),
                            time,
                        )?;
                    } else {
                        self.remove_tick_schedule(instance_id.as_str());
                    }
                }
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
            WorldEventBody::RuleDecisionRecorded(_) => {}
            WorldEventBody::ActionOverridden(_) => {}
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
            WorldEventBody::ModuleRuntimeCharged(charge) => {
                self.apply_module_runtime_charge_event(charge, time)?;
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
