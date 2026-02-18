use super::super::*;
use super::pos;
use agent_world_wasm_abi::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEffectIntent,
    ModuleEmit, ModuleOutput, ModuleSandbox, ModuleTickLifecycleDirective,
};
use agent_world_wasm_executor::FixedSandbox;
#[cfg(not(feature = "wasmtime"))]
use agent_world_wasm_executor::{WasmExecutor, WasmExecutorConfig};
use serde_json::json;
use std::collections::VecDeque;

#[test]
fn apply_module_changes_registers_and_activates() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    let wasm_bytes = b"dummy-wasm-weather";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();
    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: wasm_hash.clone(),
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["WeatherTick".to_string()],
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::PostEvent),
            filters: None,
        }],
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 2048,
            max_effects: 2,
            max_emits: 2,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    let key = ModuleRegistry::record_key(&module_manifest.module_id, &module_manifest.version);
    let record = world.module_registry().records.get(&key).unwrap();
    assert_eq!(record.manifest, module_manifest);
    assert_eq!(record.registered_by, "alice");
    assert_eq!(
        world
            .module_registry()
            .active
            .get(&module_manifest.module_id),
        Some(&module_manifest.version)
    );

    let module_events: Vec<_> = world
        .journal()
        .events
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::ModuleEvent(module_event) => Some(module_event),
            _ => None,
        })
        .collect();
    assert_eq!(module_events.len(), 2);
    assert!(matches!(
        module_events[0].kind,
        ModuleEventKind::RegisterModule { .. }
    ));
    assert!(matches!(
        module_events[1].kind,
        ModuleEventKind::ActivateModule { .. }
    ));

    if let serde_json::Value::Object(map) = &world.manifest().content {
        assert!(!map.contains_key("module_changes"));
    }
}

#[test]
fn shadow_rejects_missing_module_artifact() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: "missing-hash".to_string(),
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::ModuleChangeInvalid { .. }));
}

#[test]
fn shadow_rejects_incomplete_module_artifact_identity() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    let wasm_bytes = b"dummy-wasm-weather-identity";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: Some(ModuleArtifactIdentity {
            source_hash: String::new(),
            build_manifest_hash: "build-hash".to_string(),
            artifact_signature: "unsigned:dummy:src:build".to_string(),
        }),
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    let WorldError::ModuleChangeInvalid { reason } = err else {
        panic!("expected ModuleChangeInvalid");
    };
    assert!(reason.contains("artifact_identity is incomplete"));
}

#[test]
fn shadow_rejects_module_artifact_identity_signature_mismatch() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    let wasm_bytes = b"dummy-wasm-weather-identity-mismatch";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: Some(ModuleArtifactIdentity::unsigned(
            "different-wasm-hash",
            "src-hash",
            "build-hash",
        )),
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    let WorldError::ModuleChangeInvalid { reason } = err else {
        panic!("expected ModuleChangeInvalid");
    };
    assert!(reason.contains("artifact_identity signature mismatch"));
}

#[test]
fn shadow_rejects_unsupported_module_abi_version() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    let wasm_bytes = b"dummy-wasm-weather-abi-version";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract {
            abi_version: Some(2),
            input_schema: Some("schema.input@1".to_string()),
            output_schema: Some("schema.output@1".to_string()),
            cap_slots: Default::default(),
            policy_hooks: Vec::new(),
        },
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    let WorldError::ModuleChangeInvalid { reason } = err else {
        panic!("expected ModuleChangeInvalid");
    };
    assert!(reason.contains("abi_version unsupported"));
}

#[test]
fn shadow_rejects_partial_module_schema_contract() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    let wasm_bytes = b"dummy-wasm-weather-schema-contract";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract {
            abi_version: Some(1),
            input_schema: Some("schema.input@1".to_string()),
            output_schema: None,
            cap_slots: Default::default(),
            policy_hooks: Vec::new(),
        },
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    let WorldError::ModuleChangeInvalid { reason } = err else {
        panic!("expected ModuleChangeInvalid");
    };
    assert!(reason.contains("input_schema/output_schema pair"));
}

#[test]
fn shadow_rejects_cap_slot_binding_to_unknown_required_cap() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    let wasm_bytes = b"dummy-wasm-weather-cap-slot";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract {
            abi_version: Some(1),
            input_schema: Some("schema.input@1".to_string()),
            output_schema: Some("schema.output@1".to_string()),
            cap_slots: std::collections::BTreeMap::from([(
                "weather_api".to_string(),
                "cap.not-required".to_string(),
            )]),
            policy_hooks: Vec::new(),
        },
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    let WorldError::ModuleChangeInvalid { reason } = err else {
        panic!("expected ModuleChangeInvalid");
    };
    assert!(reason.contains("binds unknown cap_ref"));
}

#[test]
fn module_cache_loads_and_evicts() {
    let mut world = World::new();
    let wasm_a = b"module-a";
    let wasm_b = b"module-b";
    let hash_a = util::sha256_hex(wasm_a);
    let hash_b = util::sha256_hex(wasm_b);

    world
        .register_module_artifact(hash_a.clone(), wasm_a)
        .unwrap();
    world
        .register_module_artifact(hash_b.clone(), wasm_b)
        .unwrap();
    world.set_module_cache_max(1);

    let artifact_a = world.load_module(&hash_a).unwrap();
    assert_eq!(artifact_a.wasm_hash, hash_a);
    assert_eq!(artifact_a.bytes, wasm_a.to_vec());
    assert_eq!(world.module_cache_len(), 1);

    let artifact_b = world.load_module(&hash_b).unwrap();
    assert_eq!(artifact_b.wasm_hash, hash_b);
    assert_eq!(world.module_cache_len(), 1);

    let artifact_a_again = world.load_module(&hash_a).unwrap();
    assert_eq!(artifact_a_again.wasm_hash, hash_a);
    assert_eq!(world.module_cache_len(), 1);
}

#[test]
fn module_output_limits_reject_excess() {
    let world = World::new();
    let limits = ModuleLimits {
        max_mem_bytes: u64::MAX,
        max_gas: u64::MAX,
        max_call_rate: u32::MAX,
        max_output_bytes: 8,
        max_effects: 1,
        max_emits: 1,
    };

    let err = world
        .validate_module_output_limits("m.test", &limits, 2, 0, 4)
        .unwrap_err();
    assert!(matches!(err, WorldError::ModuleChangeInvalid { .. }));

    let err = world
        .validate_module_output_limits("m.test", &limits, 1, 1, 12)
        .unwrap_err();
    assert!(matches!(err, WorldError::ModuleChangeInvalid { .. }));
}

#[test]
fn module_call_queues_effects_and_emits() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    world.set_policy(PolicySet::allow_all());

    let wasm_bytes = b"module-weather";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 2,
            max_emits: 2,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    let output = ModuleOutput {
        new_state: None,
        effects: vec![ModuleEffectIntent {
            kind: "http.request".to_string(),
            params: json!({"url": "https://example.com"}),
            cap_ref: "cap.weather".to_string(),
            cap_slot: None,
        }],
        emits: vec![ModuleEmit {
            kind: "WeatherTick".to_string(),
            payload: json!({"ok": true}),
        }],
        tick_lifecycle: None,
        output_bytes: 64,
    };

    let mut sandbox = FixedSandbox::succeed(output);
    world
        .execute_module_call("m.weather", "trace-1", vec![], &mut sandbox)
        .unwrap();

    assert_eq!(world.pending_effects_len(), 1);

    let has_emit = world
        .journal()
        .events
        .iter()
        .any(|event| matches!(event.body, WorldEventBody::ModuleEmitted(_)));
    assert!(has_emit);
}

#[test]
fn module_call_resolves_effect_cap_from_cap_slot() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    world.set_policy(PolicySet::allow_all());

    let wasm_bytes = b"module-weather-cap-slot";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract {
            abi_version: Some(1),
            input_schema: Some("schema.input@1".to_string()),
            output_schema: Some("schema.output@1".to_string()),
            cap_slots: std::collections::BTreeMap::from([(
                "weather_api".to_string(),
                "cap.weather".to_string(),
            )]),
            policy_hooks: Vec::new(),
        },
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 2,
            max_emits: 0,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    let output = ModuleOutput {
        new_state: None,
        effects: vec![ModuleEffectIntent {
            kind: "http.request".to_string(),
            params: json!({"url": "https://example.com"}),
            cap_ref: String::new(),
            cap_slot: Some("weather_api".to_string()),
        }],
        emits: Vec::new(),
        tick_lifecycle: None,
        output_bytes: 64,
    };

    let mut sandbox = FixedSandbox::succeed(output);
    world
        .execute_module_call("m.weather", "trace-slot", vec![], &mut sandbox)
        .unwrap();

    let queued = world.take_next_effect().expect("queued effect");
    assert_eq!(queued.cap_ref, "cap.weather");
}

#[test]
fn module_call_rejects_effect_with_unbound_cap_slot() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    world.set_policy(PolicySet::allow_all());

    let wasm_bytes = b"module-weather-cap-slot-missing";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract {
            abi_version: Some(1),
            input_schema: Some("schema.input@1".to_string()),
            output_schema: Some("schema.output@1".to_string()),
            cap_slots: std::collections::BTreeMap::new(),
            policy_hooks: Vec::new(),
        },
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 2,
            max_emits: 0,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    let output = ModuleOutput {
        new_state: None,
        effects: vec![ModuleEffectIntent {
            kind: "http.request".to_string(),
            params: json!({"url": "https://example.com"}),
            cap_ref: String::new(),
            cap_slot: Some("missing_slot".to_string()),
        }],
        emits: Vec::new(),
        tick_lifecycle: None,
        output_bytes: 64,
    };

    let mut sandbox = FixedSandbox::succeed(output);
    let err = world
        .execute_module_call("m.weather", "trace-slot-missing", vec![], &mut sandbox)
        .unwrap_err();
    assert!(matches!(err, WorldError::ModuleCallFailed { .. }));

    let failed = world
        .journal()
        .events
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::ModuleCallFailed(failure) => Some(failure),
            _ => None,
        })
        .last()
        .expect("failure event");
    assert_eq!(failed.code, ModuleCallErrorCode::CapsDenied);
    assert!(failed.detail.contains("cap_slot not bound"));
}

#[test]
fn module_call_policy_denied_records_failure() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    world.set_policy(PolicySet {
        rules: vec![PolicyRule {
            when: PolicyWhen {
                effect_kind: Some("http.request".to_string()),
                origin_kind: None,
                cap_name: None,
            },
            decision: PolicyDecision::Deny {
                reason: "blocked".to_string(),
            },
        }],
    });

    let wasm_bytes = b"module-weather-deny";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.weather".to_string(),
        name: "Weather".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 2,
            max_emits: 2,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    let output = ModuleOutput {
        new_state: None,
        effects: vec![ModuleEffectIntent {
            kind: "http.request".to_string(),
            params: json!({"url": "https://example.com"}),
            cap_ref: "cap.weather".to_string(),
            cap_slot: None,
        }],
        emits: Vec::new(),
        tick_lifecycle: None,
        output_bytes: 64,
    };

    let mut sandbox = FixedSandbox::succeed(output);
    let err = world
        .execute_module_call("m.weather", "trace-2", vec![], &mut sandbox)
        .unwrap_err();
    assert!(matches!(err, WorldError::ModuleCallFailed { .. }));
    assert_eq!(world.pending_effects_len(), 0);

    let failed = world
        .journal()
        .events
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::ModuleCallFailed(failure) => Some(failure),
            _ => None,
        })
        .last()
        .unwrap();
    assert_eq!(failed.code, ModuleCallErrorCode::PolicyDenied);
}

struct PurePolicyHookSandbox;

impl ModuleSandbox for PurePolicyHookSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        match request.module_id.as_str() {
            "m.weather" => Ok(ModuleOutput {
                new_state: None,
                effects: vec![ModuleEffectIntent {
                    kind: "http.request".to_string(),
                    params: json!({"url": "https://example.com"}),
                    cap_ref: "cap.weather".to_string(),
                    cap_slot: None,
                }],
                emits: Vec::new(),
                tick_lifecycle: None,
                output_bytes: 64,
            }),
            "m.policy.allow" => Ok(ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "policy.allow".to_string(),
                    payload: json!({}),
                }],
                tick_lifecycle: None,
                output_bytes: 32,
            }),
            "m.policy.deny" => Ok(ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "policy.deny".to_string(),
                    payload: json!({"reason": "blocked_by_pure_policy"}),
                }],
                tick_lifecycle: None,
                output_bytes: 32,
            }),
            other => Err(ModuleCallFailure {
                module_id: request.module_id.clone(),
                trace_id: request.trace_id.clone(),
                code: ModuleCallErrorCode::Trap,
                detail: format!("unexpected module call {other}"),
            }),
        }
    }
}

fn activate_module_manifest(world: &mut World, manifest: ModuleManifest) {
    let changes = ModuleChangeSet {
        register: vec![manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: manifest.module_id.clone(),
            version: manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest_update = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world
        .propose_manifest_update(manifest_update, "alice")
        .unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();
}

#[test]
fn module_call_pure_policy_hook_allows_effect_queueing() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    world.set_policy(PolicySet::allow_all());

    let source_bytes = b"module-source-weather";
    let source_hash = util::sha256_hex(source_bytes);
    world
        .register_module_artifact(source_hash.clone(), source_bytes)
        .unwrap();
    let policy_bytes = b"module-policy-allow";
    let policy_hash = util::sha256_hex(policy_bytes);
    world
        .register_module_artifact(policy_hash.clone(), policy_bytes)
        .unwrap();

    activate_module_manifest(
        &mut world,
        ModuleManifest {
            module_id: "m.weather".to_string(),
            name: "Weather".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Reducer,
            role: ModuleRole::Domain,
            wasm_hash: source_hash,
            interface_version: "wasm-1".to_string(),
            abi_contract: ModuleAbiContract {
                abi_version: Some(1),
                input_schema: Some("schema.input@1".to_string()),
                output_schema: Some("schema.output@1".to_string()),
                cap_slots: std::collections::BTreeMap::new(),
                policy_hooks: vec!["m.policy.allow".to_string()],
            },
            exports: vec!["reduce".to_string()],
            subscriptions: Vec::new(),
            required_caps: vec!["cap.weather".to_string()],
            artifact_identity: None,
            limits: ModuleLimits {
                max_mem_bytes: 1024,
                max_gas: 10_000,
                max_call_rate: 1,
                max_output_bytes: 1024,
                max_effects: 2,
                max_emits: 0,
            },
        },
    );

    activate_module_manifest(
        &mut world,
        ModuleManifest {
            module_id: "m.policy.allow".to_string(),
            name: "PolicyAllow".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Pure,
            role: ModuleRole::Domain,
            wasm_hash: policy_hash,
            interface_version: "wasm-1".to_string(),
            abi_contract: ModuleAbiContract::default(),
            exports: vec!["call".to_string()],
            subscriptions: Vec::new(),
            required_caps: Vec::new(),
            artifact_identity: None,
            limits: ModuleLimits {
                max_mem_bytes: 1024,
                max_gas: 10_000,
                max_call_rate: 1,
                max_output_bytes: 1024,
                max_effects: 0,
                max_emits: 1,
            },
        },
    );

    let mut sandbox = PurePolicyHookSandbox;
    world
        .execute_module_call("m.weather", "trace-policy-allow", vec![], &mut sandbox)
        .unwrap();
    assert_eq!(world.pending_effects_len(), 1);
}

#[test]
fn module_call_pure_policy_hook_can_deny_effect() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap.weather"));
    world.set_policy(PolicySet::allow_all());

    let source_bytes = b"module-source-weather-deny";
    let source_hash = util::sha256_hex(source_bytes);
    world
        .register_module_artifact(source_hash.clone(), source_bytes)
        .unwrap();
    let policy_bytes = b"module-policy-deny";
    let policy_hash = util::sha256_hex(policy_bytes);
    world
        .register_module_artifact(policy_hash.clone(), policy_bytes)
        .unwrap();

    activate_module_manifest(
        &mut world,
        ModuleManifest {
            module_id: "m.weather".to_string(),
            name: "Weather".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Reducer,
            role: ModuleRole::Domain,
            wasm_hash: source_hash,
            interface_version: "wasm-1".to_string(),
            abi_contract: ModuleAbiContract {
                abi_version: Some(1),
                input_schema: Some("schema.input@1".to_string()),
                output_schema: Some("schema.output@1".to_string()),
                cap_slots: std::collections::BTreeMap::new(),
                policy_hooks: vec!["m.policy.deny".to_string()],
            },
            exports: vec!["reduce".to_string()],
            subscriptions: Vec::new(),
            required_caps: vec!["cap.weather".to_string()],
            artifact_identity: None,
            limits: ModuleLimits {
                max_mem_bytes: 1024,
                max_gas: 10_000,
                max_call_rate: 1,
                max_output_bytes: 1024,
                max_effects: 2,
                max_emits: 0,
            },
        },
    );

    activate_module_manifest(
        &mut world,
        ModuleManifest {
            module_id: "m.policy.deny".to_string(),
            name: "PolicyDeny".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Pure,
            role: ModuleRole::Domain,
            wasm_hash: policy_hash,
            interface_version: "wasm-1".to_string(),
            abi_contract: ModuleAbiContract::default(),
            exports: vec!["call".to_string()],
            subscriptions: Vec::new(),
            required_caps: Vec::new(),
            artifact_identity: None,
            limits: ModuleLimits {
                max_mem_bytes: 1024,
                max_gas: 10_000,
                max_call_rate: 1,
                max_output_bytes: 1024,
                max_effects: 0,
                max_emits: 1,
            },
        },
    );

    let mut sandbox = PurePolicyHookSandbox;
    let err = world
        .execute_module_call("m.weather", "trace-policy-deny", vec![], &mut sandbox)
        .unwrap_err();
    assert!(matches!(err, WorldError::ModuleCallFailed { .. }));
    assert_eq!(world.pending_effects_len(), 0);

    let failed = world
        .journal()
        .events
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::ModuleCallFailed(failure) => Some(failure),
            _ => None,
        })
        .last()
        .expect("failure event");
    assert_eq!(failed.code, ModuleCallErrorCode::PolicyDenied);
    assert!(failed.detail.contains("blocked_by_pure_policy"));
}

#[cfg(not(feature = "wasmtime"))]
#[test]
fn wasm_executor_skeleton_reports_unavailable() {
    let mut sandbox = WasmExecutor::new(WasmExecutorConfig::default());
    let request = ModuleCallRequest {
        module_id: "m.test".to_string(),
        wasm_hash: "hash".to_string(),
        trace_id: "trace-1".to_string(),
        entrypoint: "call".to_string(),
        input: vec![],
        limits: ModuleLimits::default(),
        wasm_bytes: Vec::new(),
    };

    let err = sandbox.call(&request).unwrap_err();
    assert_eq!(err.code, ModuleCallErrorCode::SandboxUnavailable);
    assert_eq!(err.module_id, "m.test");
    assert_eq!(err.trace_id, "trace-1");
}

#[test]
fn step_with_modules_routes_domain_events() {
    let mut world = World::new();
    world.set_policy(PolicySet::allow_all());

    let wasm_bytes = b"module-router";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.router".to_string(),
        name: "Router".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["domain.agent_registered".to_string()],
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::PostEvent),
            filters: None,
        }],
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 0,
            max_emits: 1,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });

    let output = ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: "AgentRegistered".to_string(),
            payload: json!({"ok": true}),
        }],
        tick_lifecycle: None,
        output_bytes: 64,
    };
    let mut sandbox = FixedSandbox::succeed(output);
    world.step_with_modules(&mut sandbox).unwrap();

    let has_emit = world
        .journal()
        .events
        .iter()
        .any(|event| matches!(event.body, WorldEventBody::ModuleEmitted(_)));
    assert!(has_emit);
}

#[test]
fn step_with_modules_routes_actions() {
    let mut world = World::new();
    world.set_policy(PolicySet::allow_all());

    let wasm_bytes = b"module-action-router";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.action-router".to_string(),
        name: "ActionRouter".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: vec!["action.register_agent".to_string()],
            stage: Some(ModuleSubscriptionStage::PreAction),
            filters: None,
        }],
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 0,
            max_emits: 1,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });

    let output = ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: "ActionSeen".to_string(),
            payload: json!({"agent": "agent-1"}),
        }],
        tick_lifecycle: None,
        output_bytes: 64,
    };
    let mut sandbox = FixedSandbox::succeed(output);
    world.step_with_modules(&mut sandbox).unwrap();

    let mut action_emit_index = None;
    let mut domain_event_index = None;
    for (idx, event) in world.journal().events.iter().enumerate() {
        match &event.body {
            WorldEventBody::ModuleEmitted(emit) if emit.trace_id.starts_with("action-") => {
                action_emit_index = Some(idx);
            }
            WorldEventBody::Domain(DomainEvent::AgentRegistered { agent_id, .. })
                if agent_id == "agent-1" =>
            {
                domain_event_index = Some(idx);
            }
            _ => {}
        }
    }

    let action_emit_index = action_emit_index.expect("expected action subscription emit");
    let domain_event_index = domain_event_index.expect("expected agent registration event");
    assert!(action_emit_index < domain_event_index);
}

#[derive(Default)]
struct TickLifecycleSandbox {
    calls: Vec<ModuleCallRequest>,
    outputs: VecDeque<ModuleOutput>,
}

impl TickLifecycleSandbox {
    fn with_outputs(outputs: Vec<ModuleOutput>) -> Self {
        Self {
            calls: Vec::new(),
            outputs: outputs.into(),
        }
    }
}

impl ModuleSandbox for TickLifecycleSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        self.calls.push(request.clone());
        Ok(self.outputs.pop_front().unwrap_or(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: Some(ModuleTickLifecycleDirective::Suspend),
            output_bytes: 0,
        }))
    }
}

#[test]
fn step_with_modules_routes_tick_lifecycle_with_wake_and_suspend() {
    let mut world = World::new();
    world.set_policy(PolicySet::allow_all());

    let wasm_bytes = b"module-tick-router";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.tick-router".to_string(),
        name: "TickRouter".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::Tick),
            filters: None,
        }],
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 0,
            max_emits: 0,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };
    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };
    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    let mut sandbox = TickLifecycleSandbox::with_outputs(vec![
        ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: Some(ModuleTickLifecycleDirective::WakeAfterTicks { ticks: 2 }),
            output_bytes: 0,
        },
        ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: Some(ModuleTickLifecycleDirective::Suspend),
            output_bytes: 0,
        },
    ]);

    world.step_with_modules(&mut sandbox).expect("tick 1");
    world.step_with_modules(&mut sandbox).expect("tick 2");
    world.step_with_modules(&mut sandbox).expect("tick 3");
    world.step_with_modules(&mut sandbox).expect("tick 4");

    assert_eq!(
        sandbox.calls.len(),
        2,
        "tick module should run at t=1 and t=3"
    );
    let first_input: ModuleCallInput =
        serde_cbor::from_slice(&sandbox.calls[0].input).expect("decode first tick input");
    let second_input: ModuleCallInput =
        serde_cbor::from_slice(&sandbox.calls[1].input).expect("decode second tick input");
    assert_eq!(first_input.ctx.stage.as_deref(), Some("tick"));
    assert_eq!(first_input.ctx.origin.kind, "tick");
    assert_eq!(second_input.ctx.stage.as_deref(), Some("tick"));
    assert_eq!(second_input.ctx.origin.kind, "tick");
}

#[derive(Default)]
struct CaptureEntrypointSandbox {
    entrypoints: Vec<String>,
}

impl ModuleSandbox for CaptureEntrypointSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        self.entrypoints.push(request.entrypoint.clone());
        Ok(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: None,
            output_bytes: 0,
        })
    }
}

#[test]
fn module_calls_use_entrypoint_for_kind() {
    let mut world = World::new();
    world.set_policy(PolicySet::allow_all());

    let reducer_bytes = b"module-reducer";
    let reducer_hash = util::sha256_hex(reducer_bytes);
    world
        .register_module_artifact(reducer_hash.clone(), reducer_bytes)
        .unwrap();

    let pure_bytes = b"module-pure";
    let pure_hash = util::sha256_hex(pure_bytes);
    world
        .register_module_artifact(pure_hash.clone(), pure_bytes)
        .unwrap();

    let reducer_manifest = ModuleManifest {
        module_id: "m.reducer".to_string(),
        name: "Reducer".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: reducer_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["domain.agent_registered".to_string()],
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::PostEvent),
            filters: None,
        }],
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 0,
            max_emits: 0,
        },
    };

    let pure_manifest = ModuleManifest {
        module_id: "m.pure".to_string(),
        name: "Pure".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Pure,
        role: ModuleRole::Domain,
        wasm_hash: pure_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["call".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["domain.agent_registered".to_string()],
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::PostEvent),
            filters: None,
        }],
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 1,
            max_output_bytes: 1024,
            max_effects: 0,
            max_emits: 0,
        },
    };

    let changes = ModuleChangeSet {
        register: vec![reducer_manifest.clone(), pure_manifest.clone()],
        activate: vec![
            ModuleActivation {
                module_id: reducer_manifest.module_id.clone(),
                version: reducer_manifest.version.clone(),
            },
            ModuleActivation {
                module_id: pure_manifest.module_id.clone(),
                version: pure_manifest.version.clone(),
            },
        ],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 2,
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });

    let mut sandbox = CaptureEntrypointSandbox::default();
    world.step_with_modules(&mut sandbox).unwrap();

    assert!(sandbox.entrypoints.contains(&"reduce".to_string()));
    assert!(sandbox.entrypoints.contains(&"call".to_string()));
}
