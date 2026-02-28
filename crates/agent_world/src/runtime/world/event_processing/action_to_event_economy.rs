use super::super::super::MaterialMarketQuote;
use super::*;
use std::collections::BTreeMap;

const FACTORY_DURABILITY_PPM_BASE: i64 = 1_000_000;
const FACTORY_MAINTENANCE_PART_KIND: &str = "hardware_part";
const FACTORY_MAINTENANCE_REPAIR_PPM_PER_PART: i64 = 25_000;
const FACTORY_RECYCLE_BASE_PPM: i64 = 700_000;
const BOTTLENECK_TAG_KINDS: &[&str] = &["iron_ingot", "copper_wire", "control_chip", "motor_mk1"];

impl World {
    pub(super) fn action_to_event_economy(
        &self,
        action_id: ActionId,
        action: &Action,
    ) -> Result<WorldEventBody, WorldError> {
        match action {
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
                if *kind == ResourceKind::Data
                    && from_agent_id != to_agent_id
                    && !self
                        .state
                        .has_data_access_permission(from_agent_id, to_agent_id)
                {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "data transfer denied: missing access grant owner={} grantee={}",
                                from_agent_id, to_agent_id
                            )],
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
            Action::CollectData {
                collector_agent_id,
                electricity_cost,
                data_amount,
            } => {
                if !self.state.agents.contains_key(collector_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: collector_agent_id.clone(),
                        },
                    }));
                }
                if *electricity_cost <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InvalidAmount {
                            amount: *electricity_cost,
                        },
                    }));
                }
                if *data_amount <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InvalidAmount {
                            amount: *data_amount,
                        },
                    }));
                }
                let available = self
                    .state
                    .agents
                    .get(collector_agent_id)
                    .map(|cell| cell.state.resources.get(ResourceKind::Electricity))
                    .unwrap_or(0);
                if available < *electricity_cost {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InsufficientResource {
                            agent_id: collector_agent_id.clone(),
                            kind: ResourceKind::Electricity,
                            requested: *electricity_cost,
                            available,
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::DataCollected {
                    collector_agent_id: collector_agent_id.clone(),
                    electricity_cost: *electricity_cost,
                    data_amount: *data_amount,
                }))
            }
            Action::GrantDataAccess {
                owner_agent_id,
                grantee_agent_id,
            } => {
                if !self.state.agents.contains_key(owner_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: owner_agent_id.clone(),
                        },
                    }));
                }
                if !self.state.agents.contains_key(grantee_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: grantee_agent_id.clone(),
                        },
                    }));
                }
                if owner_agent_id == grantee_agent_id {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![
                                "data access grant requires distinct owner and grantee".to_string()
                            ],
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::DataAccessGranted {
                    owner_agent_id: owner_agent_id.clone(),
                    grantee_agent_id: grantee_agent_id.clone(),
                }))
            }
            Action::RevokeDataAccess {
                owner_agent_id,
                grantee_agent_id,
            } => {
                if !self.state.agents.contains_key(owner_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: owner_agent_id.clone(),
                        },
                    }));
                }
                if !self.state.agents.contains_key(grantee_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: grantee_agent_id.clone(),
                        },
                    }));
                }
                if owner_agent_id == grantee_agent_id {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec!["data access revoke requires distinct owner and grantee"
                                .to_string()],
                        },
                    }));
                }
                Ok(WorldEventBody::Domain(DomainEvent::DataAccessRevoked {
                    owner_agent_id: owner_agent_id.clone(),
                    grantee_agent_id: grantee_agent_id.clone(),
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
            Action::MaintainFactory {
                operator_agent_id,
                factory_id,
                parts,
            } => {
                if *parts <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InvalidAmount { amount: *parts },
                    }));
                }
                if !self.state.agents.contains_key(operator_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: operator_agent_id.clone(),
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
                if factory.builder_agent_id != *operator_agent_id {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "maintain factory denied: operator {} is not builder {}",
                                operator_agent_id, factory.builder_agent_id
                            )],
                        },
                    }));
                }

                let current = factory.durability_ppm.clamp(0, FACTORY_DURABILITY_PPM_BASE);
                if current >= FACTORY_DURABILITY_PPM_BASE {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "factory {} already at full durability",
                                factory_id
                            )],
                        },
                    }));
                }

                let requested_parts = *parts;
                let consume = vec![MaterialStack::new(
                    FACTORY_MAINTENANCE_PART_KIND.to_string(),
                    requested_parts,
                )];
                let consume_ledger = self.select_material_consume_ledger_with_world_fallback(
                    factory.input_ledger.clone(),
                    &consume,
                );
                let available =
                    self.ledger_material_balance(&consume_ledger, FACTORY_MAINTENANCE_PART_KIND);
                if available < requested_parts {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::InsufficientMaterial {
                            material_kind: FACTORY_MAINTENANCE_PART_KIND.to_string(),
                            requested: requested_parts,
                            available,
                        },
                    }));
                }

                let repair_ppm = requested_parts
                    .saturating_mul(FACTORY_MAINTENANCE_REPAIR_PPM_PER_PART)
                    .max(0);
                let durability_ppm = current
                    .saturating_add(repair_ppm)
                    .clamp(0, FACTORY_DURABILITY_PPM_BASE);
                let recovered_ppm = durability_ppm.saturating_sub(current);
                if recovered_ppm <= 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "factory {} maintenance has no effect",
                                factory_id
                            )],
                        },
                    }));
                }
                let consumed_parts = recovered_ppm
                    .saturating_add(FACTORY_MAINTENANCE_REPAIR_PPM_PER_PART - 1)
                    .saturating_div(FACTORY_MAINTENANCE_REPAIR_PPM_PER_PART)
                    .clamp(1, requested_parts);

                Ok(WorldEventBody::Domain(DomainEvent::FactoryMaintained {
                    operator_agent_id: operator_agent_id.clone(),
                    factory_id: factory_id.clone(),
                    consume_ledger,
                    consumed_parts,
                    durability_ppm,
                }))
            }
            Action::RecycleFactory {
                operator_agent_id,
                factory_id,
            } => {
                if !self.state.agents.contains_key(operator_agent_id) {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentNotFound {
                            agent_id: operator_agent_id.clone(),
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
                if factory.builder_agent_id != *operator_agent_id {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "recycle factory denied: operator {} is not builder {}",
                                operator_agent_id, factory.builder_agent_id
                            )],
                        },
                    }));
                }
                let active_jobs = self
                    .state
                    .pending_recipe_jobs
                    .values()
                    .filter(|job| job.factory_id == *factory_id)
                    .count();
                if active_jobs > 0 {
                    return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::FactoryBusy {
                            factory_id: factory_id.clone(),
                            active_jobs,
                            recipe_slots: factory.spec.recipe_slots,
                        },
                    }));
                }

                let durability_ppm = factory.durability_ppm.clamp(0, FACTORY_DURABILITY_PPM_BASE);
                let recovered = factory
                    .spec
                    .build_cost
                    .iter()
                    .filter_map(|stack| {
                        if stack.amount <= 0 {
                            return None;
                        }
                        let recovered_amount = ((stack.amount as i128)
                            .saturating_mul(FACTORY_RECYCLE_BASE_PPM as i128)
                            .saturating_mul(durability_ppm as i128)
                            / (FACTORY_DURABILITY_PPM_BASE as i128)
                            / (FACTORY_DURABILITY_PPM_BASE as i128))
                            as i64;
                        if recovered_amount <= 0 {
                            None
                        } else {
                            Some(MaterialStack::new(stack.kind.clone(), recovered_amount))
                        }
                    })
                    .collect();

                Ok(WorldEventBody::Domain(DomainEvent::FactoryRecycled {
                    operator_agent_id: operator_agent_id.clone(),
                    factory_id: factory_id.clone(),
                    recycle_ledger: factory.output_ledger.clone(),
                    recovered,
                    durability_ppm,
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
                let recipe_profile = self.recipe_profile(recipe_id);
                if let Some(profile) = recipe_profile {
                    if !recipe_stage_gate_allowed(
                        self.state.industry_progress.stage,
                        profile.stage_gate.as_str(),
                    ) {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "recipe stage gate denied: recipe={} required_stage={} current_stage={}",
                                    recipe_id,
                                    profile.stage_gate,
                                    industry_stage_label(self.state.industry_progress.stage),
                                )],
                            },
                        }));
                    }
                    if !recipe_preferred_tags_compatible(
                        &profile.preferred_factory_tags,
                        &factory.spec.tags,
                    ) {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "recipe preferred_factory_tags mismatch: recipe={} preferred={:?} factory_tags={:?}",
                                    recipe_id, profile.preferred_factory_tags, factory.spec.tags
                                )],
                            },
                        }));
                    }
                }
                for stack in &plan.produce {
                    let Some(product_profile) = self.product_profile(stack.kind.as_str()) else {
                        continue;
                    };
                    if !product_unlock_stage_allowed(
                        self.state.industry_progress.stage,
                        product_profile.unlock_stage.as_str(),
                    ) {
                        return Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                            action_id,
                            reason: RejectReason::RuleDenied {
                                notes: vec![format!(
                                    "product unlock_stage denied: product={} required_stage={} current_stage={}",
                                    product_profile.product_id,
                                    product_profile.unlock_stage,
                                    industry_stage_label(self.state.industry_progress.stage),
                                )],
                            },
                        }));
                    }
                }
                let preferred_consume_ledger = factory.input_ledger.clone();
                let consume_ledger = self.select_material_consume_ledger_with_world_fallback(
                    preferred_consume_ledger.clone(),
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
                let market_quotes =
                    build_material_market_quotes(self, &preferred_consume_ledger, &plan.consume);
                let bottleneck_tags = resolve_recipe_bottleneck_tags(recipe_profile, &plan.consume);
                let scarcity_delay_ticks = compute_local_scarcity_delay_ticks(
                    self,
                    &preferred_consume_ledger,
                    &consume_ledger,
                    &plan.consume,
                    &bottleneck_tags,
                );
                let duration_ticks = plan
                    .duration_ticks
                    .max(1)
                    .saturating_add(scarcity_delay_ticks);
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
                    bottleneck_tags,
                    market_quotes,
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
            _ => unreachable!("action_to_event_economy received unsupported action variant"),
        }
    }
}

fn infer_bottleneck_tags(consume: &[MaterialStack]) -> Vec<String> {
    let tags: BTreeSet<String> = consume
        .iter()
        .filter_map(|stack| {
            let normalized = stack.kind.to_ascii_lowercase();
            BOTTLENECK_TAG_KINDS
                .iter()
                .find(|kind| normalized == **kind)
                .map(|kind| (*kind).to_string())
        })
        .collect();
    tags.into_iter().collect()
}

fn compute_local_scarcity_delay_ticks(
    world: &World,
    preferred_consume_ledger: &MaterialLedgerId,
    consume_ledger: &MaterialLedgerId,
    consume: &[MaterialStack],
    bottleneck_tags: &[String],
) -> u32 {
    if *preferred_consume_ledger == MaterialLedgerId::world()
        || *consume_ledger != MaterialLedgerId::world()
        || bottleneck_tags.is_empty()
    {
        return 0;
    }

    let bottleneck_set: BTreeSet<&str> = bottleneck_tags.iter().map(String::as_str).collect();
    let mut requested_total: i128 = 0;
    let mut deficit_total: i128 = 0;
    for stack in consume {
        if stack.amount <= 0 {
            continue;
        }
        let normalized_kind = stack.kind.to_ascii_lowercase();
        if !bottleneck_set.contains(normalized_kind.as_str()) {
            continue;
        }
        let requested = stack.amount as i128;
        let available = world
            .ledger_material_balance(preferred_consume_ledger, stack.kind.as_str())
            .max(0) as i128;
        requested_total = requested_total.saturating_add(requested);
        deficit_total = deficit_total.saturating_add(requested.saturating_sub(available).max(0));
    }

    if requested_total <= 0 || deficit_total <= 0 {
        return 0;
    }
    let deficit_ratio_bps = deficit_total
        .saturating_mul(10_000)
        .saturating_div(requested_total);
    if deficit_ratio_bps >= 7_000 {
        2
    } else {
        1
    }
}

fn governance_tax_bps_for_material_quotes(world: &World) -> u16 {
    world
        .state
        .gameplay_policy
        .electricity_tax_bps
        .saturating_add(world.state.gameplay_policy.data_tax_bps)
        .min(10_000)
}

fn build_material_market_quotes(
    world: &World,
    preferred_consume_ledger: &MaterialLedgerId,
    consume: &[MaterialStack],
) -> Vec<MaterialMarketQuote> {
    let mut requested_by_kind: BTreeMap<String, i64> = BTreeMap::new();
    for stack in consume {
        if stack.amount <= 0 {
            continue;
        }
        let entry = requested_by_kind.entry(stack.kind.clone()).or_insert(0);
        *entry = entry.saturating_add(stack.amount);
    }
    if requested_by_kind.is_empty() {
        return Vec::new();
    }

    let governance_tax_bps = governance_tax_bps_for_material_quotes(world);
    let mut quotes = Vec::with_capacity(requested_by_kind.len());
    for (kind, requested_amount) in requested_by_kind {
        let transit_loss_bps = material_transit_loss_bps_for_kind(world, kind.as_str());
        let local_available_amount = world
            .ledger_material_balance(preferred_consume_ledger, kind.as_str())
            .max(0);
        let world_available_amount = world
            .ledger_material_balance(&MaterialLedgerId::world(), kind.as_str())
            .max(0);
        let local_deficit_amount = requested_amount
            .saturating_sub(local_available_amount)
            .max(0);
        let deficit_ratio_bps = if requested_amount > 0 {
            ((local_deficit_amount as i128)
                .saturating_mul(10_000)
                .saturating_div(requested_amount as i128)) as i64
        } else {
            0
        };
        let effective_cost_index_ppm = 1_000_000_i64
            .saturating_add(deficit_ratio_bps.saturating_mul(100))
            .saturating_add(transit_loss_bps.saturating_mul(100))
            .saturating_add(i64::from(governance_tax_bps).saturating_mul(100));
        quotes.push(MaterialMarketQuote {
            kind,
            requested_amount,
            local_available_amount,
            world_available_amount,
            local_deficit_amount,
            transit_loss_bps,
            governance_tax_bps,
            effective_cost_index_ppm,
        });
    }
    quotes
}

fn recipe_stage_gate_allowed(
    current_stage: crate::runtime::IndustryStage,
    stage_gate: &str,
) -> bool {
    let normalized = stage_gate.trim();
    if normalized.is_empty() {
        return true;
    }
    let Some(required_stage) = parse_industry_stage(normalized) else {
        return true;
    };
    current_stage >= required_stage
}

fn product_unlock_stage_allowed(
    current_stage: crate::runtime::IndustryStage,
    unlock_stage: &str,
) -> bool {
    recipe_stage_gate_allowed(current_stage, unlock_stage)
}

fn parse_industry_stage(raw: &str) -> Option<crate::runtime::IndustryStage> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "bootstrap" => Some(crate::runtime::IndustryStage::Bootstrap),
        "scale_out" | "scaleout" | "scale-out" => Some(crate::runtime::IndustryStage::ScaleOut),
        "governance" => Some(crate::runtime::IndustryStage::Governance),
        _ => None,
    }
}

fn industry_stage_label(stage: crate::runtime::IndustryStage) -> &'static str {
    match stage {
        crate::runtime::IndustryStage::Bootstrap => "bootstrap",
        crate::runtime::IndustryStage::ScaleOut => "scale_out",
        crate::runtime::IndustryStage::Governance => "governance",
    }
}

fn recipe_preferred_tags_compatible(preferred_tags: &[String], factory_tags: &[String]) -> bool {
    if preferred_tags.is_empty() {
        return true;
    }
    let normalized_factory: BTreeSet<String> = factory_tags
        .iter()
        .map(|tag| tag.trim().to_ascii_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect();
    preferred_tags.iter().any(|tag| {
        let normalized = tag.trim().to_ascii_lowercase();
        !normalized.is_empty() && normalized_factory.contains(normalized.as_str())
    })
}

fn resolve_recipe_bottleneck_tags(
    recipe_profile: Option<&crate::runtime::RecipeProfileV1>,
    consume: &[MaterialStack],
) -> Vec<String> {
    let from_profile: BTreeSet<String> = recipe_profile
        .map(|profile| {
            profile
                .bottleneck_tags
                .iter()
                .map(|tag| tag.trim().to_ascii_lowercase())
                .filter(|tag| !tag.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if !from_profile.is_empty() {
        return from_profile.into_iter().collect();
    }
    infer_bottleneck_tags(consume)
}

fn material_transit_loss_bps_for_kind(world: &World, kind: &str) -> i64 {
    let base = MATERIAL_TRANSFER_LOSS_PER_KM_BPS.max(0);
    let factor = world
        .material_profile(kind)
        .map(|profile| match profile.transport_loss_class {
            crate::runtime::MaterialTransportLossClass::Low => 1_i64,
            crate::runtime::MaterialTransportLossClass::Medium => 2_i64,
            crate::runtime::MaterialTransportLossClass::High => 4_i64,
        })
        .unwrap_or(1);
    base.saturating_mul(factor)
}
