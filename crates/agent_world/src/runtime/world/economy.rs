use agent_world_wasm_abi::{
    FactoryBuildDecision, FactoryBuildRequest, MaterialStack, ModuleCallErrorCode,
    ModuleCallFailure, ModuleCallInput, ModuleCallOrigin, ModuleContext, ModuleKind, ModuleOutput,
    ModuleSandbox, ProductValidationDecision, ProductValidationRequest, RecipeExecutionPlan,
    RecipeExecutionRequest,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;

use super::super::util::to_canonical_cbor;
use super::super::{
    Action, ActionEnvelope, ActionId, DomainEvent, MaterialLedgerId, RejectReason, WorldError,
    WorldEvent, WorldEventBody, M4_PRODUCT_CONTROL_CHIP_MODULE_ID, M4_PRODUCT_IRON_INGOT_MODULE_ID,
    M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID, M4_PRODUCT_MOTOR_MODULE_ID,
};
use super::World;
use crate::simulator::ResourceKind;

const FACTORY_BUILD_DECISION_EMIT_KIND: &str = "economy.factory_build_decision";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";

#[derive(Debug, Clone, Serialize)]
struct ProductValidationModuleCallRequest {
    product_id: String,
    stack: MaterialStack,
    deterministic_seed: u64,
    kind: String,
    amount: i64,
}

pub(super) enum EconomyActionResolution {
    Resolved(Action),
    Rejected(RejectReason),
}

impl World {
    // ---------------------------------------------------------------------
    // Economy runtime helpers
    // ---------------------------------------------------------------------

    pub fn pending_factory_builds_len(&self) -> usize {
        self.state.pending_factory_builds.len()
    }

    pub fn pending_recipe_jobs_len(&self) -> usize {
        self.state.pending_recipe_jobs.len()
    }

    pub fn has_factory(&self, factory_id: &str) -> bool {
        self.state.factories.contains_key(factory_id)
    }

    pub(super) fn resolve_module_backed_economy_action(
        &mut self,
        envelope: &ActionEnvelope,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<EconomyActionResolution, WorldError> {
        match &envelope.action {
            Action::BuildFactoryWithModule {
                builder_agent_id,
                site_id,
                module_id,
                spec,
            } => {
                if module_id.trim().is_empty() {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec!["factory module_id cannot be empty".to_string()],
                        },
                    ));
                }
                let preferred_ledger = MaterialLedgerId::agent(builder_agent_id.clone());
                let request_ledger = self.select_material_consume_ledger_for_module_request(
                    preferred_ledger,
                    &spec.build_cost,
                );
                let request = FactoryBuildRequest {
                    factory_id: spec.factory_id.clone(),
                    site_id: site_id.clone(),
                    builder: builder_agent_id.clone(),
                    available_inputs: spec
                        .build_cost
                        .iter()
                        .map(|stack| {
                            MaterialStack::new(
                                stack.kind.clone(),
                                self.ledger_material_balance(&request_ledger, stack.kind.as_str()),
                            )
                        })
                        .collect(),
                    available_inputs_by_ledger: Some(self.material_stacks_by_ledger()),
                    available_power: self.resource_balance(ResourceKind::Electricity),
                };
                let decision = self.evaluate_factory_build_with_module(
                    module_id.as_str(),
                    envelope.id,
                    &request,
                    sandbox,
                )?;
                if !decision.accepted {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec![format!(
                                "factory module denied: {}",
                                decision
                                    .reject_reason
                                    .as_deref()
                                    .unwrap_or("build rejected")
                            )],
                        },
                    ));
                }

                let mut resolved_spec = spec.clone();
                if !decision.consume.is_empty() {
                    resolved_spec.build_cost = decision.consume;
                }
                if decision.duration_ticks > 0 {
                    resolved_spec.build_time_ticks = decision.duration_ticks;
                }

                Ok(EconomyActionResolution::Resolved(Action::BuildFactory {
                    builder_agent_id: builder_agent_id.clone(),
                    site_id: site_id.clone(),
                    spec: resolved_spec,
                }))
            }
            Action::ScheduleRecipeWithModule {
                requester_agent_id,
                factory_id,
                recipe_id,
                module_id,
                desired_batches,
                deterministic_seed,
            } => {
                if module_id.trim().is_empty() {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec!["recipe module_id cannot be empty".to_string()],
                        },
                    ));
                }
                if *desired_batches == 0 {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec!["desired_batches must be > 0".to_string()],
                        },
                    ));
                }
                let preferred_ledger = self
                    .state
                    .factories
                    .get(factory_id)
                    .map(|factory| factory.input_ledger.clone())
                    .unwrap_or_else(MaterialLedgerId::world);
                let mut available_inputs = self.ledger_material_stacks(&preferred_ledger);
                if available_inputs.is_empty() && preferred_ledger != MaterialLedgerId::world() {
                    available_inputs = self.material_stacks();
                }

                let request = RecipeExecutionRequest {
                    recipe_id: recipe_id.clone(),
                    factory_id: factory_id.clone(),
                    desired_batches: *desired_batches,
                    available_inputs,
                    available_inputs_by_ledger: Some(self.material_stacks_by_ledger()),
                    available_power: self.resource_balance(ResourceKind::Electricity),
                    deterministic_seed: *deterministic_seed,
                };
                let plan = self.evaluate_recipe_with_module(
                    module_id.as_str(),
                    envelope.id,
                    &request,
                    sandbox,
                )?;

                if let Some(reason) = &plan.reject_reason {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec![format!("recipe module denied: {reason}")],
                        },
                    ));
                }

                Ok(EconomyActionResolution::Resolved(Action::ScheduleRecipe {
                    requester_agent_id: requester_agent_id.clone(),
                    factory_id: factory_id.clone(),
                    recipe_id: recipe_id.clone(),
                    plan,
                }))
            }
            Action::ValidateProductWithModule {
                requester_agent_id,
                module_id,
                stack,
                deterministic_seed,
            } => {
                if module_id.trim().is_empty() {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec!["product module_id cannot be empty".to_string()],
                        },
                    ));
                }
                if stack.amount <= 0 {
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied {
                            notes: vec!["product stack amount must be > 0".to_string()],
                        },
                    ));
                }
                let request = ProductValidationRequest {
                    product_id: stack.kind.clone(),
                    stack: stack.clone(),
                    deterministic_seed: *deterministic_seed,
                };
                let decision = self.evaluate_product_with_module(
                    module_id.as_str(),
                    envelope.id,
                    &request,
                    sandbox,
                )?;
                if !decision.accepted {
                    let notes = if decision.notes.is_empty() {
                        vec![format!("product module denied: {}", decision.product_id)]
                    } else {
                        decision
                            .notes
                            .iter()
                            .map(|note| format!("product module denied: {note}"))
                            .collect()
                    };
                    return Ok(EconomyActionResolution::Rejected(
                        RejectReason::RuleDenied { notes },
                    ));
                }
                Ok(EconomyActionResolution::Resolved(Action::ValidateProduct {
                    requester_agent_id: requester_agent_id.clone(),
                    module_id: module_id.clone(),
                    stack: stack.clone(),
                    decision,
                }))
            }
            _ => Ok(EconomyActionResolution::Resolved(envelope.action.clone())),
        }
    }

    pub(super) fn process_due_economy_jobs(&mut self) -> Result<Vec<WorldEvent>, WorldError> {
        let now = self.state.time;
        let mut emitted = Vec::new();

        let mut due_builds: Vec<_> = self
            .state
            .pending_factory_builds
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_builds.sort_by_key(|job| (job.ready_at, job.job_id));

        for job in due_builds {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::FactoryBuilt {
                    job_id: job.job_id,
                    builder_agent_id: job.builder_agent_id,
                    site_id: job.site_id,
                    spec: job.spec,
                }),
                None,
            )?;
            if let Some(event) = self.journal.events.last() {
                emitted.push(event.clone());
            }
        }

        let mut due_recipes: Vec<_> = self
            .state
            .pending_recipe_jobs
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_recipes.sort_by_key(|job| (job.ready_at, job.job_id));

        for job in due_recipes {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::RecipeCompleted {
                    job_id: job.job_id,
                    requester_agent_id: job.requester_agent_id,
                    factory_id: job.factory_id,
                    recipe_id: job.recipe_id,
                    accepted_batches: job.accepted_batches,
                    produce: job.produce,
                    byproducts: job.byproducts,
                    output_ledger: job.output_ledger,
                }),
                None,
            )?;
            if let Some(event) = self.journal.events.last() {
                emitted.push(event.clone());
            }
        }

        Ok(emitted)
    }

    pub(super) fn process_due_economy_jobs_with_modules(
        &mut self,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<Vec<WorldEvent>, WorldError> {
        let now = self.state.time;
        let mut emitted = Vec::new();

        let mut due_builds: Vec<_> = self
            .state
            .pending_factory_builds
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_builds.sort_by_key(|job| (job.ready_at, job.job_id));

        for job in due_builds {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::FactoryBuilt {
                    job_id: job.job_id,
                    builder_agent_id: job.builder_agent_id,
                    site_id: job.site_id,
                    spec: job.spec,
                }),
                None,
            )?;
            if let Some(event) = self.journal.events.last() {
                emitted.push(event.clone());
            }
        }

        let mut due_recipes: Vec<_> = self
            .state
            .pending_recipe_jobs
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_recipes.sort_by_key(|job| (job.ready_at, job.job_id));

        for job in due_recipes {
            let mut committed_produce = job.produce.clone();
            let mut committed_byproducts = job.byproducts.clone();

            for (index, stack) in job.produce.iter().enumerate() {
                let Some(module_id) = self.resolve_product_module_for_stack(stack.kind.as_str())
                else {
                    continue;
                };
                let request = ProductValidationRequest {
                    product_id: stack.kind.clone(),
                    stack: stack.clone(),
                    deterministic_seed: job
                        .job_id
                        .wrapping_mul(1_000_003)
                        .wrapping_add(index as u64)
                        .wrapping_add(self.state.time),
                };
                let decision = self.evaluate_product_with_module(
                    module_id.as_str(),
                    job.job_id,
                    &request,
                    sandbox,
                )?;
                let validation_event = self.action_to_event(&ActionEnvelope {
                    id: job.job_id,
                    action: Action::ValidateProduct {
                        requester_agent_id: job.requester_agent_id.clone(),
                        module_id,
                        stack: stack.clone(),
                        decision,
                    },
                })?;
                let rejected = matches!(
                    validation_event,
                    WorldEventBody::Domain(DomainEvent::ActionRejected { .. })
                );
                self.append_event(validation_event, None)?;
                if let Some(event) = self.journal.events.last() {
                    emitted.push(event.clone());
                }
                if rejected {
                    committed_produce.clear();
                    committed_byproducts.clear();
                    break;
                }
            }

            self.append_event(
                WorldEventBody::Domain(DomainEvent::RecipeCompleted {
                    job_id: job.job_id,
                    requester_agent_id: job.requester_agent_id,
                    factory_id: job.factory_id,
                    recipe_id: job.recipe_id,
                    accepted_batches: job.accepted_batches,
                    produce: committed_produce,
                    byproducts: committed_byproducts,
                    output_ledger: job.output_ledger,
                }),
                None,
            )?;
            if let Some(event) = self.journal.events.last() {
                emitted.push(event.clone());
            }
        }

        Ok(emitted)
    }

    fn evaluate_factory_build_with_module(
        &mut self,
        module_id: &str,
        action_id: ActionId,
        request: &FactoryBuildRequest,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<FactoryBuildDecision, WorldError> {
        let trace_id = format!("action-{action_id}-{module_id}-factory");
        let output = self.execute_economy_module_call(module_id, &trace_id, request, sandbox)?;
        if !output.effects.is_empty() {
            return self.economy_module_output_invalid(
                module_id,
                &trace_id,
                "factory module output must not contain effects",
            );
        }
        self.extract_economy_emit(
            module_id,
            &trace_id,
            &output,
            FACTORY_BUILD_DECISION_EMIT_KIND,
        )
    }

    fn evaluate_recipe_with_module(
        &mut self,
        module_id: &str,
        action_id: ActionId,
        request: &RecipeExecutionRequest,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<RecipeExecutionPlan, WorldError> {
        let trace_id = format!("action-{action_id}-{module_id}-recipe");
        let output = self.execute_economy_module_call(module_id, &trace_id, request, sandbox)?;
        if !output.effects.is_empty() {
            return self.economy_module_output_invalid(
                module_id,
                &trace_id,
                "recipe module output must not contain effects",
            );
        }
        self.extract_economy_emit(
            module_id,
            &trace_id,
            &output,
            RECIPE_EXECUTION_PLAN_EMIT_KIND,
        )
    }

    fn evaluate_product_with_module(
        &mut self,
        module_id: &str,
        action_id: ActionId,
        request: &ProductValidationRequest,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<ProductValidationDecision, WorldError> {
        let trace_id = format!("action-{action_id}-{module_id}-product");
        // Keep backward compatibility for product modules that decode legacy stack-only payloads.
        let wire_request = ProductValidationModuleCallRequest {
            product_id: request.product_id.clone(),
            stack: request.stack.clone(),
            deterministic_seed: request.deterministic_seed,
            kind: request.stack.kind.clone(),
            amount: request.stack.amount,
        };
        let output =
            self.execute_economy_module_call(module_id, &trace_id, &wire_request, sandbox)?;
        if !output.effects.is_empty() {
            return self.economy_module_output_invalid(
                module_id,
                &trace_id,
                "product module output must not contain effects",
            );
        }
        self.extract_economy_emit(module_id, &trace_id, &output, PRODUCT_VALIDATION_EMIT_KIND)
    }

    fn resolve_product_module_for_stack(&self, product_kind: &str) -> Option<String> {
        if let Some(module_id) = Self::builtin_product_module_for_kind(product_kind) {
            if self.module_registry.active.contains_key(module_id) {
                return Some(module_id.to_string());
            }
        }
        let suffix = format!(".{product_kind}");
        self.module_registry
            .active
            .keys()
            .find(|module_id| {
                module_id.contains(".product.") && module_id.ends_with(suffix.as_str())
            })
            .cloned()
    }

    fn builtin_product_module_for_kind(product_kind: &str) -> Option<&'static str> {
        match product_kind {
            "iron_ingot" => Some(M4_PRODUCT_IRON_INGOT_MODULE_ID),
            "control_chip" => Some(M4_PRODUCT_CONTROL_CHIP_MODULE_ID),
            "motor_mk1" => Some(M4_PRODUCT_MOTOR_MODULE_ID),
            "logistics_drone" => Some(M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID),
            _ => None,
        }
    }

    fn execute_economy_module_call<T: Serialize>(
        &mut self,
        module_id: &str,
        trace_id: &str,
        request: &T,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<ModuleOutput, WorldError> {
        let manifest = self.active_module_manifest(module_id)?.clone();
        let world_config_hash = self.current_manifest_hash()?;
        let action_bytes = to_canonical_cbor(request)?;
        let ctx = ModuleContext {
            v: "wasm-1".to_string(),
            module_id: module_id.to_string(),
            trace_id: trace_id.to_string(),
            time: self.state.time,
            origin: ModuleCallOrigin {
                kind: "action".to_string(),
                id: trace_id.to_string(),
            },
            limits: manifest.limits.clone(),
            world_config_hash: Some(world_config_hash),
        };
        let state = match manifest.kind {
            ModuleKind::Reducer => Some(
                self.state
                    .module_states
                    .get(module_id)
                    .cloned()
                    .unwrap_or_default(),
            ),
            ModuleKind::Pure => None,
        };
        let input = ModuleCallInput {
            ctx,
            event: None,
            action: Some(action_bytes),
            state,
        };
        let input_bytes = to_canonical_cbor(&input)?;
        self.execute_module_call(module_id, trace_id.to_string(), input_bytes, sandbox)
    }

    fn extract_economy_emit<T: DeserializeOwned>(
        &mut self,
        module_id: &str,
        trace_id: &str,
        output: &ModuleOutput,
        expected_emit_kind: &str,
    ) -> Result<T, WorldError> {
        let mut payload = None;
        for emit in &output.emits {
            if emit.kind != expected_emit_kind {
                continue;
            }
            if payload.is_some() {
                return self.economy_module_output_invalid(
                    module_id,
                    trace_id,
                    format!("multiple {expected_emit_kind} emits in module output"),
                );
            }
            payload = Some(emit.payload.clone());
        }
        let Some(payload) = payload else {
            return self.economy_module_output_invalid(
                module_id,
                trace_id,
                format!("missing {expected_emit_kind} emit in module output"),
            );
        };
        match serde_json::from_value(payload) {
            Ok(parsed) => Ok(parsed),
            Err(err) => self.economy_module_output_invalid(
                module_id,
                trace_id,
                format!("decode {expected_emit_kind} failed: {err}"),
            ),
        }
    }

    fn economy_module_output_invalid<T>(
        &mut self,
        module_id: &str,
        trace_id: &str,
        detail: impl Into<String>,
    ) -> Result<T, WorldError> {
        let failure = ModuleCallFailure {
            module_id: module_id.to_string(),
            trace_id: trace_id.to_string(),
            code: ModuleCallErrorCode::InvalidOutput,
            detail: detail.into(),
        };
        self.append_event(WorldEventBody::ModuleCallFailed(failure.clone()), None)?;
        Err(WorldError::ModuleCallFailed {
            module_id: failure.module_id,
            trace_id: failure.trace_id,
            code: failure.code,
            detail: failure.detail,
        })
    }

    fn material_stacks(&self) -> Vec<MaterialStack> {
        self.ledger_material_stacks(&MaterialLedgerId::world())
    }

    fn material_stacks_by_ledger(&self) -> BTreeMap<String, Vec<MaterialStack>> {
        self.state
            .material_ledgers
            .keys()
            .map(|ledger_id| {
                (
                    ledger_id.to_string(),
                    self.ledger_material_stacks(ledger_id),
                )
            })
            .collect()
    }

    fn select_material_consume_ledger_for_module_request(
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
}
