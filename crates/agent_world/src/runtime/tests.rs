//! Tests for the runtime module.

use super::*;
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn pos(lat: f64, lon: f64) -> crate::geometry::GeoPos {
    crate::geometry::GeoPos {
        lat_deg: lat,
        lon_deg: lon,
    }
}

#[test]
fn register_and_move_agent() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(1.0, 1.0),
    });
    world.step().unwrap();

    let agent = world.state().agents.get("agent-1").unwrap();
    assert_eq!(agent.state.pos, pos(1.0, 1.0));
    assert_eq!(world.journal().len(), 2);
}

#[test]
fn snapshot_and_replay() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    let snapshot = world.snapshot();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(2.0, 2.0),
    });
    world.step().unwrap();

    let journal = world.journal().clone();
    let restored = World::from_snapshot(snapshot, journal).unwrap();
    assert_eq!(restored.state(), world.state());
}

#[test]
fn rejects_invalid_actions() {
    let mut world = World::new();
    let action_id = world.submit_action(Action::MoveAgent {
        agent_id: "missing".to_string(),
        to: pos(1.0, 1.0),
    });
    world.step().unwrap();

    let event = world.journal().events.last().unwrap();
    match &event.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { action_id: id, reason }) => {
            assert_eq!(*id, action_id);
            assert!(matches!(reason, RejectReason::AgentNotFound { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn effect_pipeline_signs_receipt() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
    world.set_policy(PolicySet::allow_all());
    world.set_receipt_signer(ReceiptSigner::hmac_sha256(b"secret"));

    let intent_id = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com"}),
            "cap_all",
            EffectOrigin::System,
        )
        .unwrap();

    let intent = world.take_next_effect().unwrap();
    assert_eq!(intent.intent_id, intent_id);

    let receipt = EffectReceipt {
        intent_id: intent_id.clone(),
        status: "ok".to_string(),
        payload: json!({"status": 200}),
        cost_cents: Some(5),
        signature: None,
    };

    world.ingest_receipt(receipt).unwrap();

    let event = world.journal().events.last().unwrap();
    match &event.body {
        WorldEventBody::ReceiptAppended(receipt) => {
            assert!(receipt.signature.is_some());
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn policy_denies_effect() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
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

    let err = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com"}),
            "cap_all",
            EffectOrigin::System,
        )
        .unwrap_err();

    assert!(matches!(err, WorldError::PolicyDenied { .. }));

    let event = world.journal().events.last().unwrap();
    match &event.body {
        WorldEventBody::PolicyDecisionRecorded(record) => {
            assert!(matches!(record.decision, PolicyDecision::Deny { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn governance_flow_applies_manifest() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "demo" }),
    };

    let proposal_id = world
        .propose_manifest_update(manifest.clone(), "alice")
        .unwrap();
    let shadow_hash = world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    let applied_hash = world.apply_proposal(proposal_id).unwrap();

    assert_eq!(shadow_hash, applied_hash);
    assert_eq!(world.manifest().version, 2);
    assert_eq!(world.manifest().content, manifest.content);
}

#[test]
fn governance_patch_updates_manifest() {
    let mut world = World::new();
    let base_hash = world.current_manifest_hash().unwrap();
    let patch = ManifestPatch {
        base_manifest_hash: base_hash,
        ops: vec![ManifestPatchOp::Set {
            path: vec!["settings".to_string(), "mode".to_string()],
            value: json!("fast"),
        }],
        new_version: Some(3),
    };

    let proposal_id = world
        .propose_manifest_patch(patch, "alice")
        .unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    assert_eq!(world.manifest().version, 3);
    assert_eq!(world.manifest().content, json!({ "settings": { "mode": "fast" } }));
}

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
        wasm_hash: wasm_hash.clone(),
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["WeatherTick".to_string()],
            action_kinds: Vec::new(),
            filters: None,
        }],
        required_caps: vec!["cap.weather".to_string()],
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

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .unwrap();
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
        wasm_hash: "missing-hash".to_string(),
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
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

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .unwrap();
    let err = world.shadow_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::ModuleChangeInvalid { .. }));
}

#[test]
fn module_cache_loads_and_evicts() {
    let mut world = World::new();
    let wasm_a = b"module-a";
    let wasm_b = b"module-b";
    let hash_a = util::sha256_hex(wasm_a);
    let hash_b = util::sha256_hex(wasm_b);

    world.register_module_artifact(hash_a.clone(), wasm_a).unwrap();
    world.register_module_artifact(hash_b.clone(), wasm_b).unwrap();
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
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
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

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .unwrap();
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
        }],
        emits: vec![ModuleEmit {
            kind: "WeatherTick".to_string(),
            payload: json!({"ok": true}),
        }],
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
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: vec!["cap.weather".to_string()],
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

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .unwrap();
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
        }],
        emits: Vec::new(),
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
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["domain.agent_registered".to_string()],
            action_kinds: Vec::new(),
            filters: None,
        }],
        required_caps: Vec::new(),
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

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .unwrap();
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
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: vec!["action.register_agent".to_string()],
            filters: None,
        }],
        required_caps: Vec::new(),
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

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .unwrap();
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

#[test]
fn manifest_diff_and_merge() {
    let base = Manifest {
        version: 1,
        content: json!({ "a": 1, "b": { "c": 2 } }),
    };
    let target = Manifest {
        version: 2,
        content: json!({ "a": 1, "b": { "c": 3 }, "d": 4 }),
    };

    let patch = diff_manifest(&base, &target).unwrap();
    let applied = apply_manifest_patch(&base, &patch).unwrap();
    assert_eq!(applied, target);

    let base_hash = util::hash_json(&base).unwrap();
    let patch1 = ManifestPatch {
        base_manifest_hash: base_hash.clone(),
        ops: vec![ManifestPatchOp::Set {
            path: vec!["b".to_string(), "c".to_string()],
            value: json!(3),
        }],
        new_version: Some(2),
    };
    let patch2 = ManifestPatch {
        base_manifest_hash: base_hash,
        ops: vec![ManifestPatchOp::Set {
            path: vec!["e".to_string()],
            value: json!(5),
        }],
        new_version: Some(3),
    };

    let merged = merge_manifest_patches(&base, &[patch1, patch2]).unwrap();
    let merged_applied = apply_manifest_patch(&base, &merged).unwrap();
    let expected = Manifest {
        version: 3,
        content: json!({ "a": 1, "b": { "c": 3 }, "e": 5 }),
    };
    assert_eq!(merged_applied, expected);
}

#[test]
fn merge_reports_conflicts() {
    let base = Manifest {
        version: 1,
        content: json!({ "a": { "b": 1 }, "x": 1 }),
    };
    let base_hash = util::hash_json(&base).unwrap();
    let patch1 = ManifestPatch {
        base_manifest_hash: base_hash.clone(),
        ops: vec![ManifestPatchOp::Set {
            path: vec!["a".to_string(), "b".to_string()],
            value: json!(2),
        }],
        new_version: None,
    };
    let patch2 = ManifestPatch {
        base_manifest_hash: base_hash,
        ops: vec![ManifestPatchOp::Set {
            path: vec!["a".to_string()],
            value: json!({ "b": 3 }),
        }],
        new_version: None,
    };

    let result = merge_manifest_patches_with_conflicts(&base, &[patch1, patch2]).unwrap();
    assert_eq!(result.conflicts.len(), 1);
    assert_eq!(result.conflicts[0].path, vec!["a".to_string()]);
    assert_eq!(result.conflicts[0].kind, ConflictKind::PrefixOverlap);
    assert_eq!(result.conflicts[0].patches, vec![0, 1]);
    assert_eq!(result.conflicts[0].ops.len(), 2);
}

#[test]
fn persist_and_restore_world() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("agent-world-{unique}"));

    world.save_to_dir(&dir).unwrap();

    let restored = World::load_from_dir(&dir).unwrap();
    assert_eq!(restored.state(), world.state());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rollback_to_snapshot_resets_state() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    let snapshot = world.snapshot();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(9.0, 9.0),
    });
    world.step().unwrap();
    assert_eq!(world.state().agents.get("agent-1").unwrap().state.pos, pos(9.0, 9.0));

    let journal = world.journal().clone();
    world
        .rollback_to_snapshot(snapshot.clone(), journal, "test-rollback")
        .unwrap();

    assert_eq!(world.state(), &snapshot.state);
    let last = world.journal().events.last().unwrap();
    assert!(matches!(last.body, WorldEventBody::RollbackApplied(_)));
}

#[test]
fn snapshot_retention_policy_prunes_old_entries() {
    let mut world = World::new();
    world.set_snapshot_retention(SnapshotRetentionPolicy { max_snapshots: 1 });

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    let snap1 = world.create_snapshot().unwrap();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(3.0, 3.0),
    });
    world.step().unwrap();
    let snap2 = world.create_snapshot().unwrap();

    assert_eq!(world.snapshot_catalog().records.len(), 1);
    let last_record = &world.snapshot_catalog().records[0];
    assert_eq!(last_record.snapshot_hash, util::hash_json(&snap2).unwrap());
    assert_ne!(last_record.snapshot_hash, util::hash_json(&snap1).unwrap());
}

#[test]
fn snapshot_file_pruning_removes_old_files() {
    let mut world = World::new();
    world.set_snapshot_retention(SnapshotRetentionPolicy { max_snapshots: 1 });

    let dir = std::env::temp_dir().join(format!(
        "agent-world-snapshots-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    world.save_snapshot_to_dir(&dir).unwrap();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    world.save_snapshot_to_dir(&dir).unwrap();

    let snapshots_dir = dir.join("snapshots");
    let file_count = fs::read_dir(&snapshots_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .count();
    assert_eq!(file_count, 1);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn audit_filter_by_kind_and_cause() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
    world.set_policy(PolicySet::allow_all());

    let intent_id = world
        .emit_effect(
            "http.request",
            json!({ "url": "https://example.com" }),
            "cap_all",
            EffectOrigin::System,
        )
        .unwrap();

    let intent = world.take_next_effect().unwrap();
    assert_eq!(intent.intent_id, intent_id);

    let receipt = EffectReceipt {
        intent_id: intent_id.clone(),
        status: "ok".to_string(),
        payload: json!({ "status": 200 }),
        cost_cents: None,
        signature: None,
    };
    world.ingest_receipt(receipt).unwrap();

    let filter = AuditFilter {
        kinds: Some(vec![AuditEventKind::ReceiptAppended]),
        caused_by: Some(AuditCausedBy::Effect),
        ..AuditFilter::default()
    };
    let events = world.audit_events(&filter);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].caused_by,
        Some(CausedBy::Effect { .. })
    ));
}

#[test]
fn audit_log_export_writes_file() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let dir = std::env::temp_dir().join(format!(
        "agent-world-audit-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("audit.json");

    world
        .save_audit_log(&path, &AuditFilter::default())
        .unwrap();
    let events: Vec<WorldEvent> = util::read_json_from_path(&path).unwrap();
    assert!(!events.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn scheduler_round_robin() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        pos: pos(1.0, 1.0),
    });
    world.step().unwrap();

    let first = world.schedule_next().unwrap();
    assert_eq!(first.agent_id, "agent-1");
    let second = world.schedule_next().unwrap();
    assert_eq!(second.agent_id, "agent-2");
    assert!(world.schedule_next().is_none());
}
