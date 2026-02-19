use agent_world_wasm_abi::{
    ModuleCallInput, ModuleCallOrigin, ModuleSandbox, ModuleSubscriptionStage,
    ModuleTickLifecycleDirective,
};

use super::super::util::to_canonical_cbor;
use super::super::{ModuleKind, ModuleManifest, ModuleRegistry, WorldError};
use super::module_runtime_labels::{
    module_kind_label, module_role_label, subscription_stage_label,
};
use super::World;
use crate::simulator::ModuleInstallTarget;

impl World {
    pub(super) fn sync_tick_schedule_for_activation(
        &mut self,
        module_id: &str,
        version: &str,
        time: u64,
    ) -> Result<(), WorldError> {
        let key = ModuleRegistry::record_key(module_id, version);
        let record = self.module_registry.records.get(&key).ok_or_else(|| {
            WorldError::ModuleChangeInvalid {
                reason: format!("module record missing {key}"),
            }
        })?;
        if module_has_tick_subscription(&record.manifest) {
            self.module_tick_schedule
                .insert(module_id.to_string(), time);
        } else {
            self.module_tick_schedule.remove(module_id);
        }
        Ok(())
    }

    pub(super) fn remove_tick_schedule(&mut self, module_id: &str) {
        self.module_tick_schedule.remove(module_id);
    }

    pub fn route_tick_to_modules(
        &mut self,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<usize, WorldError> {
        let now = self.state.time;
        let mut module_ids: Vec<String> = self
            .module_tick_schedule
            .iter()
            .filter_map(|(module_id, wake_at)| (*wake_at <= now).then_some(module_id.clone()))
            .collect();
        module_ids.sort();
        if module_ids.is_empty() {
            return Ok(0);
        }

        let world_config_hash = self.current_manifest_hash()?;
        let mut invoked = 0;
        for module_id in module_ids {
            // Always remove the previous schedule first. The module output decides whether to
            // reschedule itself (wake) or stay suspended.
            self.module_tick_schedule.remove(module_id.as_str());

            let manifest = match self.active_module_manifest(module_id.as_str()) {
                Ok(manifest) => manifest.clone(),
                Err(_) => continue,
            };
            if !module_has_tick_subscription(&manifest) {
                continue;
            }

            let install_target = self
                .state
                .installed_module_targets
                .get(&module_id)
                .cloned()
                .unwrap_or(ModuleInstallTarget::SelfAgent);
            let (origin_kind, origin_id, trace_id) = match install_target {
                ModuleInstallTarget::SelfAgent => (
                    "tick".to_string(),
                    now.to_string(),
                    format!("tick-{}-{}", now, module_id),
                ),
                ModuleInstallTarget::LocationInfrastructure { location_id } => {
                    let location_id = location_id.trim().to_string();
                    if location_id.is_empty() {
                        (
                            "tick".to_string(),
                            now.to_string(),
                            format!("tick-{}-{}", now, module_id),
                        )
                    } else {
                        (
                            "infrastructure_tick".to_string(),
                            format!("{}:{}", location_id, now),
                            format!("infra-tick-{}-{}-{}", now, location_id, module_id),
                        )
                    }
                }
            };
            let state = match manifest.kind {
                ModuleKind::Reducer => Some(
                    self.state
                        .module_states
                        .get(&module_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                ModuleKind::Pure => None,
            };
            let input = ModuleCallInput {
                ctx: agent_world_wasm_abi::ModuleContext {
                    v: "wasm-1".to_string(),
                    module_id: module_id.clone(),
                    trace_id: trace_id.clone(),
                    time: now,
                    origin: ModuleCallOrigin {
                        kind: origin_kind,
                        id: origin_id,
                    },
                    limits: manifest.limits.clone(),
                    stage: Some(
                        subscription_stage_label(ModuleSubscriptionStage::Tick).to_string(),
                    ),
                    world_config_hash: Some(world_config_hash.clone()),
                    manifest_hash: Some(world_config_hash.clone()),
                    journal_height: Some(self.journal.events.len() as u64),
                    module_version: Some(manifest.version.clone()),
                    module_kind: Some(module_kind_label(&manifest.kind).to_string()),
                    module_role: Some(module_role_label(&manifest.role).to_string()),
                },
                event: None,
                action: None,
                state,
            };
            let input_bytes = to_canonical_cbor(&input)?;
            let output = self.execute_module_call(&module_id, trace_id, input_bytes, sandbox)?;
            invoked += 1;

            match output.tick_lifecycle {
                Some(ModuleTickLifecycleDirective::WakeAfterTicks { ticks }) => {
                    let wake_after = ticks.max(1);
                    self.module_tick_schedule
                        .insert(module_id, now.saturating_add(wake_after));
                }
                Some(ModuleTickLifecycleDirective::Suspend) | None => {}
            }
        }
        Ok(invoked)
    }
}

fn module_has_tick_subscription(manifest: &ModuleManifest) -> bool {
    manifest
        .subscriptions
        .iter()
        .any(|subscription| subscription.resolved_stage() == ModuleSubscriptionStage::Tick)
}
