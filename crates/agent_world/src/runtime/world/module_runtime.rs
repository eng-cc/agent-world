use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value as JsonValue;

use super::World;
use super::super::{
    Action, ActionEnvelope, DomainEvent, EffectOrigin, ModuleArtifact, ModuleCallErrorCode,
    ModuleCallFailure, ModuleCallInput, ModuleCallOrigin, ModuleCallRequest, ModuleContext,
    ModuleEmitEvent, ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest,
    ModuleRegistry, ModuleSubscription, ModuleSubscriptionStage, WorldError, WorldEvent,
    WorldEventBody,
};
use super::super::util::to_canonical_cbor;

impl World {
    // ---------------------------------------------------------------------
    // Module artifact and limits
    // ---------------------------------------------------------------------

    pub fn register_module_artifact(
        &mut self,
        wasm_hash: impl Into<String>,
        bytes: &[u8],
    ) -> Result<(), WorldError> {
        let wasm_hash = wasm_hash.into();
        let computed = super::super::util::sha256_hex(bytes);
        if computed != wasm_hash {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("artifact hash mismatch expected {wasm_hash} found {computed}"),
            });
        }
        self.module_artifacts.insert(wasm_hash);
        self.module_artifact_bytes
            .insert(computed, bytes.to_vec());
        Ok(())
    }

    pub fn set_module_limits_max(&mut self, limits: ModuleLimits) {
        self.module_limits_max = limits;
    }

    pub fn set_module_cache_max(&mut self, max_cached_modules: usize) {
        self.module_cache.set_max_cached_modules(max_cached_modules);
    }

    pub fn load_module(&mut self, wasm_hash: &str) -> Result<ModuleArtifact, WorldError> {
        if let Some(artifact) = self.module_cache.get(wasm_hash) {
            return Ok(artifact);
        }
        let bytes = self
            .module_artifact_bytes
            .get(wasm_hash)
            .ok_or_else(|| WorldError::ModuleChangeInvalid {
                reason: format!("module artifact bytes missing {wasm_hash}"),
            })?
            .clone();
        let artifact = ModuleArtifact {
            wasm_hash: wasm_hash.to_string(),
            bytes,
        };
        self.module_cache.insert(artifact.clone());
        Ok(artifact)
    }

    pub fn validate_module_output_limits(
        &self,
        module_id: &str,
        limits: &ModuleLimits,
        effect_count: usize,
        emit_count: usize,
        output_bytes: u64,
    ) -> Result<(), WorldError> {
        if effect_count as u32 > limits.max_effects {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module output effects exceeded {module_id}"),
            });
        }
        if emit_count as u32 > limits.max_emits {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module output emits exceeded {module_id}"),
            });
        }
        if output_bytes > limits.max_output_bytes {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module output bytes exceeded {module_id}"),
            });
        }
        Ok(())
    }

    pub fn execute_module_call(
        &mut self,
        module_id: &str,
        trace_id: impl Into<String>,
        input: Vec<u8>,
        sandbox: &mut dyn super::super::ModuleSandbox,
    ) -> Result<super::super::ModuleOutput, WorldError> {
        let trace_id = trace_id.into();
        let manifest = self.active_module_manifest(module_id)?.clone();
        let wasm_hash = manifest.wasm_hash.clone();
        let artifact = self.load_module(&wasm_hash)?;

        let request = ModuleCallRequest {
            module_id: module_id.to_string(),
            wasm_hash,
            trace_id: trace_id.clone(),
            entrypoint: manifest.kind.entrypoint().to_string(),
            input,
            limits: manifest.limits.clone(),
            wasm_bytes: artifact.bytes,
        };

        let output = match sandbox.call(&request) {
            Ok(output) => output,
            Err(failure) => {
                self.append_event(
                    WorldEventBody::ModuleCallFailed(failure.clone()),
                    None,
                )?;
                return Err(WorldError::ModuleCallFailed {
                    module_id: failure.module_id,
                    trace_id: failure.trace_id,
                    code: failure.code,
                    detail: failure.detail,
                });
            }
        };

        self.process_module_output(module_id, &trace_id, &manifest, &output)?;
        Ok(output)
    }

    pub fn route_event_to_modules(
        &mut self,
        event: &WorldEvent,
        sandbox: &mut dyn super::super::ModuleSandbox,
    ) -> Result<usize, WorldError> {
        let event_kind = event_kind_label(&event.body);
        let event_value = serde_json::to_value(event)?;
        let mut module_ids: Vec<String> =
            self.module_registry.active.keys().cloned().collect();
        module_ids.sort();
        let event_bytes = to_canonical_cbor(event)?;
        let world_config_hash = self.current_manifest_hash()?;
        let mut invoked = 0;
        for module_id in module_ids {
            let (subscribed, manifest) = {
                let version = self
                    .module_registry
                    .active
                    .get(&module_id)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module not active {module_id}"),
                    })?;
                let key = ModuleRegistry::record_key(&module_id, version);
                let record = self
                    .module_registry
                    .records
                    .get(&key)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module record missing {key}"),
                    })?;
                let manifest = record.manifest.clone();
                let subscribed = module_subscribes_to_event(
                    &manifest.subscriptions,
                    event_kind,
                    &event_value,
                );
                (subscribed, manifest)
            };
            if !subscribed {
                continue;
            }

            let trace_id = format!("event-{}-{}", event.id, module_id);
            let ctx = ModuleContext {
                v: "wasm-1".to_string(),
                module_id: module_id.clone(),
                trace_id: trace_id.clone(),
                time: event.time,
                origin: ModuleCallOrigin {
                    kind: "event".to_string(),
                    id: event.id.to_string(),
                },
                limits: manifest.limits.clone(),
                world_config_hash: Some(world_config_hash.clone()),
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
                ctx,
                event: Some(event_bytes.clone()),
                action: None,
                state,
            };
            let input_bytes = to_canonical_cbor(&input)?;
            self.execute_module_call(&module_id, trace_id, input_bytes, sandbox)?;
            invoked += 1;
        }
        Ok(invoked)
    }

    pub fn route_action_to_modules(
        &mut self,
        envelope: &ActionEnvelope,
        sandbox: &mut dyn super::super::ModuleSandbox,
    ) -> Result<usize, WorldError> {
        self.route_action_to_modules_with_stage(
            envelope,
            ModuleSubscriptionStage::PreAction,
            sandbox,
        )
    }

    pub fn route_action_to_modules_with_stage(
        &mut self,
        envelope: &ActionEnvelope,
        stage: ModuleSubscriptionStage,
        sandbox: &mut dyn super::super::ModuleSandbox,
    ) -> Result<usize, WorldError> {
        let action_kind = action_kind_label(&envelope.action);
        let action_value = serde_json::to_value(envelope)?;
        let mut module_ids: Vec<String> =
            self.module_registry.active.keys().cloned().collect();
        module_ids.sort();
        let action_bytes = to_canonical_cbor(envelope)?;
        let world_config_hash = self.current_manifest_hash()?;
        let mut invoked = 0;

        for module_id in module_ids {
            let (subscribed, manifest) = {
                let version = self
                    .module_registry
                    .active
                    .get(&module_id)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module not active {module_id}"),
                    })?;
                let key = ModuleRegistry::record_key(&module_id, version);
                let record = self
                    .module_registry
                    .records
                    .get(&key)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module record missing {key}"),
                    })?;
                let manifest = record.manifest.clone();
                let subscribed = module_subscribes_to_action(
                    &manifest.subscriptions,
                    stage,
                    action_kind,
                    &action_value,
                );
                (subscribed, manifest)
            };
            if !subscribed {
                continue;
            }

            let trace_id = format!("action-{}-{}", envelope.id, module_id);
            let ctx = ModuleContext {
                v: "wasm-1".to_string(),
                module_id: module_id.clone(),
                trace_id: trace_id.clone(),
                time: self.state.time,
                origin: ModuleCallOrigin {
                    kind: "action".to_string(),
                    id: envelope.id.to_string(),
                },
                limits: manifest.limits.clone(),
                world_config_hash: Some(world_config_hash.clone()),
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
                ctx,
                event: None,
                action: Some(action_bytes.clone()),
                state,
            };
            let input_bytes = to_canonical_cbor(&input)?;
            self.execute_module_call(&module_id, trace_id, input_bytes, sandbox)?;
            invoked += 1;
        }

        Ok(invoked)
    }

    pub(super) fn validate_module_changes(
        &self,
        changes: &super::super::ModuleChangeSet,
    ) -> Result<(), WorldError> {
        let mut register_ids = BTreeSet::new();
        let mut activate_ids = BTreeSet::new();
        let mut deactivate_ids = BTreeSet::new();
        let mut upgrade_ids = BTreeSet::new();

        for module in &changes.register {
            if !register_ids.insert(module.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate register module_id {}", module.module_id),
                });
            }
        }

        for activation in &changes.activate {
            if !activate_ids.insert(activation.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate activate module_id {}", activation.module_id),
                });
            }
        }

        for deactivation in &changes.deactivate {
            if !deactivate_ids.insert(deactivation.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate deactivate module_id {}", deactivation.module_id),
                });
            }
        }

        for upgrade in &changes.upgrade {
            if !upgrade_ids.insert(upgrade.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate upgrade module_id {}", upgrade.module_id),
                });
            }
            if upgrade.manifest.module_id != upgrade.module_id {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "upgrade manifest module_id mismatch {}",
                        upgrade.module_id
                    ),
                });
            }
        }

        let mut planned_records = BTreeSet::new();
        for module in &changes.register {
            let key = ModuleRegistry::record_key(&module.module_id, &module.version);
            if self.module_registry.records.contains_key(&key) || !planned_records.insert(key.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("module version already registered {key}"),
                });
            }
        }

        for upgrade in &changes.upgrade {
            let to_key = ModuleRegistry::record_key(&upgrade.module_id, &upgrade.to_version);
            if self.module_registry.records.contains_key(&to_key) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("module version already registered {to_key}"),
                });
            }

            let from_key = ModuleRegistry::record_key(&upgrade.module_id, &upgrade.from_version);
            if !self.module_registry.records.contains_key(&from_key) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("upgrade source missing {from_key}"),
                });
            }

            if let Some(active_version) = self.module_registry.active.get(&upgrade.module_id) {
                if active_version != &upgrade.from_version {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!(
                            "upgrade source version mismatch for {} (active {})",
                            upgrade.module_id, active_version
                        ),
                    });
                }
            }
        }

        for activation in &changes.activate {
            let key = ModuleRegistry::record_key(&activation.module_id, &activation.version);
            let exists = self.module_registry.records.contains_key(&key)
                || planned_records.contains(&key);
            if !exists {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("activate target missing {key}"),
                });
            }
        }

        let mut will_activate = BTreeSet::new();
        for activation in &changes.activate {
            will_activate.insert(activation.module_id.clone());
        }
        for deactivation in &changes.deactivate {
            let has_active = self
                .module_registry
                .active
                .contains_key(&deactivation.module_id);
            if !has_active && !will_activate.contains(&deactivation.module_id) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "deactivate target not active {}",
                        deactivation.module_id
                    ),
                });
            }
        }

        Ok(())
    }

    pub(super) fn shadow_validate_module_changes(
        &self,
        changes: &super::super::ModuleChangeSet,
    ) -> Result<(), WorldError> {
        for module in &changes.register {
            self.validate_module_manifest(module)?;
        }
        for upgrade in &changes.upgrade {
            self.validate_module_manifest(&upgrade.manifest)?;
        }
        Ok(())
    }

    fn validate_module_manifest(&self, module: &ModuleManifest) -> Result<(), WorldError> {
        if module.module_id.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: "module_id is empty".to_string(),
            });
        }
        if module.version.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module version missing for {}", module.module_id),
            });
        }
        if module.wasm_hash.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module wasm_hash missing for {}", module.module_id),
            });
        }
        if module.interface_version.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module interface_version missing for {}", module.module_id),
            });
        }
        if !self.module_artifacts.contains(&module.wasm_hash) {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module artifact missing {}", module.wasm_hash),
            });
        }

        if module.interface_version != "wasm-1" {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module interface_version unsupported {}",
                    module.interface_version
                ),
            });
        }

        let expected_export = module.kind.entrypoint();
        if !module.exports.iter().any(|name| name == expected_export) {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module exports missing {} for {}",
                    expected_export, module.module_id
                ),
            });
        }

        for subscription in &module.subscriptions {
            validate_subscription_filters(&subscription.filters, &module.module_id)?;
        }

        self.validate_module_limits(&module.module_id, &module.limits)?;

        for cap in &module.required_caps {
            let grant = self.capabilities.get(cap).ok_or_else(|| {
                WorldError::ModuleChangeInvalid {
                    reason: format!("module cap missing {cap}"),
                }
            })?;
            if grant.is_expired(self.state.time) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("module cap expired {cap}"),
                });
            }
        }

        Ok(())
    }

    fn validate_module_limits(
        &self,
        module_id: &str,
        limits: &ModuleLimits,
    ) -> Result<(), WorldError> {
        let max = &self.module_limits_max;
        if limits.max_mem_bytes > max.max_mem_bytes {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_mem_bytes exceeded for {module_id}"),
            });
        }
        if limits.max_gas > max.max_gas {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_gas exceeded for {module_id}"),
            });
        }
        if limits.max_call_rate > max.max_call_rate {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_call_rate exceeded for {module_id}"),
            });
        }
        if limits.max_output_bytes > max.max_output_bytes {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_output_bytes exceeded for {module_id}"),
            });
        }
        if limits.max_effects > max.max_effects {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_effects exceeded for {module_id}"),
            });
        }
        if limits.max_emits > max.max_emits {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_emits exceeded for {module_id}"),
            });
        }
        Ok(())
    }

    fn active_module_manifest(&self, module_id: &str) -> Result<&ModuleManifest, WorldError> {
        let version = self
            .module_registry
            .active
            .get(module_id)
            .ok_or_else(|| WorldError::ModuleChangeInvalid {
                reason: format!("module not active {module_id}"),
            })?;
        let key = ModuleRegistry::record_key(module_id, version);
        let record = self
            .module_registry
            .records
            .get(&key)
            .ok_or_else(|| WorldError::ModuleChangeInvalid {
                reason: format!("module record missing {key}"),
            })?;
        Ok(&record.manifest)
    }

    fn module_call_failed(&mut self, failure: ModuleCallFailure) -> Result<(), WorldError> {
        self.append_event(WorldEventBody::ModuleCallFailed(failure.clone()), None)?;
        Err(WorldError::ModuleCallFailed {
            module_id: failure.module_id,
            trace_id: failure.trace_id,
            code: failure.code,
            detail: failure.detail,
        })
    }

    pub(super) fn apply_module_changes(
        &mut self,
        proposal_id: super::super::ProposalId,
        changes: &super::super::ModuleChangeSet,
        actor: &str,
    ) -> Result<(), WorldError> {
        let mut registers = changes.register.clone();
        registers.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for module in registers {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::RegisterModule {
                    module,
                    registered_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        let mut upgrades = changes.upgrade.clone();
        upgrades.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for upgrade in upgrades {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::UpgradeModule {
                    module_id: upgrade.module_id,
                    from_version: upgrade.from_version,
                    to_version: upgrade.to_version,
                    wasm_hash: upgrade.manifest.wasm_hash.clone(),
                    manifest: upgrade.manifest,
                    upgraded_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        let mut activations = changes.activate.clone();
        activations.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for activation in activations {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::ActivateModule {
                    module_id: activation.module_id,
                    version: activation.version,
                    activated_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        let mut deactivations = changes.deactivate.clone();
        deactivations.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for deactivation in deactivations {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::DeactivateModule {
                    module_id: deactivation.module_id,
                    reason: deactivation.reason,
                    deactivated_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        Ok(())
    }

    pub(super) fn apply_module_event(
        &mut self,
        event: &ModuleEvent,
        time: super::super::WorldTime,
    ) -> Result<(), WorldError> {
        match &event.kind {
            ModuleEventKind::RegisterModule { module, registered_by } => {
                let key = ModuleRegistry::record_key(&module.module_id, &module.version);
                self.module_registry.records.insert(
                    key,
                    super::super::ModuleRecord {
                        manifest: module.clone(),
                        registered_at: time,
                        registered_by: registered_by.clone(),
                        audit_event_id: None,
                    },
                );
                self.module_artifacts.insert(module.wasm_hash.clone());
            }
            ModuleEventKind::UpgradeModule {
                module_id,
                to_version,
                manifest,
                upgraded_by,
                ..
            } => {
                let key = ModuleRegistry::record_key(module_id, to_version);
                self.module_registry.records.insert(
                    key,
                    super::super::ModuleRecord {
                        manifest: manifest.clone(),
                        registered_at: time,
                        registered_by: upgraded_by.clone(),
                        audit_event_id: None,
                    },
                );
                self.module_artifacts.insert(manifest.wasm_hash.clone());
            }
            ModuleEventKind::ActivateModule { module_id, version, .. } => {
                self.module_registry
                    .active
                    .insert(module_id.clone(), version.clone());
            }
            ModuleEventKind::DeactivateModule { module_id, .. } => {
                self.module_registry.active.remove(module_id);
            }
        }
        Ok(())
    }

    fn process_module_output(
        &mut self,
        module_id: &str,
        trace_id: &str,
        manifest: &ModuleManifest,
        output: &super::super::ModuleOutput,
    ) -> Result<(), WorldError> {
        if manifest.kind == ModuleKind::Pure && output.new_state.is_some() {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::InvalidOutput,
                detail: "pure module returned new_state".to_string(),
            });
        }
        if output.effects.len() as u32 > manifest.limits.max_effects {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::EffectLimitExceeded,
                detail: "effects exceeded".to_string(),
            });
        }
        if output.emits.len() as u32 > manifest.limits.max_emits {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::EmitLimitExceeded,
                detail: "emits exceeded".to_string(),
            });
        }
        if output.output_bytes > manifest.limits.max_output_bytes {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::OutputTooLarge,
                detail: "output bytes exceeded".to_string(),
            });
        }

        for effect in &output.effects {
            if !manifest.required_caps.iter().any(|cap| cap == &effect.cap_ref) {
                return self.module_call_failed(ModuleCallFailure {
                    module_id: module_id.to_string(),
                    trace_id: trace_id.to_string(),
                    code: ModuleCallErrorCode::CapsDenied,
                    detail: format!("cap_ref not allowed {}", effect.cap_ref),
                });
            }
        }

        let mut intents = Vec::new();
        for effect in &output.effects {
            let intent = match self.build_effect_intent(
                effect.kind.clone(),
                effect.params.clone(),
                effect.cap_ref.clone(),
                EffectOrigin::Module {
                    module_id: module_id.to_string(),
                },
            ) {
                Ok(intent) => intent,
                Err(err) => {
                    let (code, detail) = match err {
                        WorldError::CapabilityMissing { cap_ref } => {
                            (ModuleCallErrorCode::CapsDenied, format!("cap missing {cap_ref}"))
                        }
                        WorldError::CapabilityExpired { cap_ref } => (
                            ModuleCallErrorCode::CapsDenied,
                            format!("cap expired {cap_ref}"),
                        ),
                        WorldError::CapabilityNotAllowed { cap_ref, kind } => (
                            ModuleCallErrorCode::CapsDenied,
                            format!("cap not allowed {cap_ref} {kind}"),
                        ),
                        WorldError::PolicyDenied { reason, .. } => {
                            (ModuleCallErrorCode::PolicyDenied, reason)
                        }
                        other => (ModuleCallErrorCode::InvalidOutput, format!("{other:?}")),
                    };
                    return self.module_call_failed(ModuleCallFailure {
                        module_id: module_id.to_string(),
                        trace_id: trace_id.to_string(),
                        code,
                        detail,
                    });
                }
            };
            intents.push(intent);
        }

        if let Some(state) = &output.new_state {
            let update = super::super::ModuleStateUpdate {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                state: state.clone(),
            };
            self.append_event(WorldEventBody::ModuleStateUpdated(update), None)?;
        }

        for intent in intents {
            self.append_event(WorldEventBody::EffectQueued(intent), None)?;
        }

        for emit in &output.emits {
            let event = ModuleEmitEvent {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                kind: emit.kind.clone(),
                payload: emit.payload.clone(),
            };
            self.append_event(WorldEventBody::ModuleEmitted(event), None)?;
        }

        Ok(())
    }
}

fn event_kind_label(body: &WorldEventBody) -> &'static str {
    match body {
        WorldEventBody::Domain(event) => match event {
            DomainEvent::AgentRegistered { .. } => "domain.agent_registered",
            DomainEvent::AgentMoved { .. } => "domain.agent_moved",
            DomainEvent::ActionRejected { .. } => "domain.action_rejected",
        },
        WorldEventBody::EffectQueued(_) => "effect.queued",
        WorldEventBody::ReceiptAppended(_) => "effect.receipt_appended",
        WorldEventBody::PolicyDecisionRecorded(_) => "policy.decision_recorded",
        WorldEventBody::Governance(_) => "governance",
        WorldEventBody::ModuleEvent(_) => "module.event",
        WorldEventBody::ModuleCallFailed(_) => "module.call_failed",
        WorldEventBody::ModuleEmitted(_) => "module.emitted",
        WorldEventBody::ModuleStateUpdated(_) => "module.state_updated",
        WorldEventBody::SnapshotCreated(_) => "snapshot.created",
        WorldEventBody::ManifestUpdated(_) => "manifest.updated",
        WorldEventBody::RollbackApplied(_) => "rollback.applied",
    }
}

fn action_kind_label(action: &Action) -> &'static str {
    match action {
        Action::RegisterAgent { .. } => "action.register_agent",
        Action::MoveAgent { .. } => "action.move_agent",
    }
}

fn module_subscribes_to_event(
    subscriptions: &[ModuleSubscription],
    event_kind: &str,
    event_value: &JsonValue,
) -> bool {
    subscriptions.iter().any(|subscription| {
        subscription.stage == ModuleSubscriptionStage::PostEvent
            && subscription
                .event_kinds
                .iter()
                .any(|pattern| subscription_match(pattern, event_kind))
            && subscription_filters_match(&subscription.filters, FilterKind::Event, event_value)
    })
}

fn module_subscribes_to_action(
    subscriptions: &[ModuleSubscription],
    stage: ModuleSubscriptionStage,
    action_kind: &str,
    action_value: &JsonValue,
) -> bool {
    subscriptions.iter().any(|subscription| {
        subscription.stage == stage
            && subscription
                .action_kinds
                .iter()
                .any(|pattern| subscription_match(pattern, action_kind))
            && subscription_filters_match(&subscription.filters, FilterKind::Action, action_value)
    })
}

fn subscription_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len().saturating_sub(1)];
        return value.starts_with(prefix);
    }
    pattern == value
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubscriptionFilters {
    #[serde(default)]
    event: Option<RuleSet>,
    #[serde(default)]
    action: Option<RuleSet>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RuleSet {
    List(Vec<MatchRule>),
    Group(RuleGroup),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleGroup {
    #[serde(default)]
    all: Vec<MatchRule>,
    #[serde(default)]
    any: Vec<MatchRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchRule {
    path: String,
    #[serde(default)]
    eq: Option<JsonValue>,
    #[serde(default)]
    ne: Option<JsonValue>,
    #[serde(default)]
    gt: Option<f64>,
    #[serde(default)]
    gte: Option<f64>,
    #[serde(default)]
    lt: Option<f64>,
    #[serde(default)]
    lte: Option<f64>,
    #[serde(default)]
    re: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum FilterKind {
    Event,
    Action,
}

fn subscription_filters_match(
    filters: &Option<JsonValue>,
    kind: FilterKind,
    value: &JsonValue,
) -> bool {
    let Some(filters_value) = filters else {
        return true;
    };
    if filters_value.is_null() {
        return true;
    }
    let parsed: SubscriptionFilters = match serde_json::from_value(filters_value.clone()) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };
    let rules = match kind {
        FilterKind::Event => parsed.event.as_ref(),
        FilterKind::Action => parsed.action.as_ref(),
    };
    let Some(rules) = rules else {
        return true;
    };
    ruleset_matches(rules, value)
}

fn validate_subscription_filters(
    filters: &Option<JsonValue>,
    module_id: &str,
) -> Result<(), WorldError> {
    let Some(filters_value) = filters else {
        return Ok(());
    };
    if filters_value.is_null() {
        return Ok(());
    }
    let parsed: SubscriptionFilters =
        serde_json::from_value(filters_value.clone()).map_err(|err| {
            WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module {module_id} subscription filters invalid: {err}"
                ),
            }
        })?;
    for ruleset in parsed.event.iter().chain(parsed.action.iter()) {
        validate_ruleset(ruleset, module_id)?;
    }
    Ok(())
}

fn ruleset_matches(ruleset: &RuleSet, value: &JsonValue) -> bool {
    match ruleset {
        RuleSet::List(rules) => rules.iter().all(|rule| match_rule(rule, value)),
        RuleSet::Group(group) => {
            let all_ok = group.all.iter().all(|rule| match_rule(rule, value));
            if !all_ok {
                return false;
            }
            if group.any.is_empty() {
                return true;
            }
            group.any.iter().any(|rule| match_rule(rule, value))
        }
    }
}

fn match_rule(rule: &MatchRule, value: &JsonValue) -> bool {
    let Some(current) = value.pointer(&rule.path) else {
        return false;
    };
    if let Some(expected) = &rule.eq {
        return current == expected;
    }
    if let Some(expected) = &rule.ne {
        return current != expected;
    }
    if let Some(pattern) = &rule.re {
        let Some(text) = current.as_str() else {
            return false;
        };
        return regex::Regex::new(pattern)
            .map(|re| re.is_match(text))
            .unwrap_or(false);
    }
    if let Some(threshold) = rule.gt {
        return compare_number(current, |value| value > threshold);
    }
    if let Some(threshold) = rule.gte {
        return compare_number(current, |value| value >= threshold);
    }
    if let Some(threshold) = rule.lt {
        return compare_number(current, |value| value < threshold);
    }
    if let Some(threshold) = rule.lte {
        return compare_number(current, |value| value <= threshold);
    }
    false
}

fn compare_number<F>(value: &JsonValue, predicate: F) -> bool
where
    F: Fn(f64) -> bool,
{
    value.as_f64().map(predicate).unwrap_or(false)
}

fn validate_ruleset(ruleset: &RuleSet, module_id: &str) -> Result<(), WorldError> {
    match ruleset {
        RuleSet::List(rules) => {
            for rule in rules {
                validate_rule(rule, module_id)?;
            }
        }
        RuleSet::Group(group) => {
            for rule in group.all.iter().chain(group.any.iter()) {
                validate_rule(rule, module_id)?;
            }
        }
    }
    Ok(())
}

fn validate_rule(rule: &MatchRule, module_id: &str) -> Result<(), WorldError> {
    if !rule.path.is_empty() && !rule.path.starts_with('/') {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "module {module_id} subscription filter path must start with '/': {}",
                rule.path
            ),
        });
    }

    let mut operators = 0usize;
    operators += usize::from(rule.eq.is_some());
    operators += usize::from(rule.ne.is_some());
    operators += usize::from(rule.gt.is_some());
    operators += usize::from(rule.gte.is_some());
    operators += usize::from(rule.lt.is_some());
    operators += usize::from(rule.lte.is_some());
    operators += usize::from(rule.re.is_some());
    if operators != 1 {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "module {module_id} subscription filter must specify exactly one operator"
            ),
        });
    }

    for number in [
        rule.gt,
        rule.gte,
        rule.lt,
        rule.lte,
    ]
    .into_iter()
    .flatten()
    {
        if !number.is_finite() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module {module_id} subscription filter numeric value must be finite"
                ),
            });
        }
    }

    if let Some(pattern) = &rule.re {
        if regex::Regex::new(pattern).is_err() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module {module_id} subscription filter regex invalid"
                ),
            });
        }
    }

    Ok(())
}
