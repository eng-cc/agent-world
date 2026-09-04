use super::super::*;
use super::modules::activate_module_manifest;
use super::signed_test_artifact_identity;
use oasis7_wasm_abi::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleEffectIntent, ModuleEmit,
    ModuleOutput, ModuleSandbox,
};

const EFFECT_CAP: &str = "cap.route.effect";

#[derive(Clone)]
struct RouteSandbox {
    output: ModuleOutput,
    calls: usize,
    fail_after_first: bool,
}

impl RouteSandbox {
    fn new(output: ModuleOutput, fail_after_first: bool) -> Self {
        Self {
            output,
            calls: 0,
            fail_after_first,
        }
    }
}

impl ModuleSandbox for RouteSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let call = self.calls;
        self.calls = self.calls.saturating_add(1);
        if self.fail_after_first && call > 0 {
            return Err(ModuleCallFailure {
                module_id: request.module_id.clone(),
                trace_id: request.trace_id.clone(),
                code: ModuleCallErrorCode::Trap,
                detail: "late route failure after first module publication".to_string(),
            });
        }
        Ok(self.output.clone())
    }
}

fn route_manifest(module_id: &str, wasm_hash: &str) -> ModuleManifest {
    ModuleManifest {
        module_id: module_id.to_string(),
        name: module_id.to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: wasm_hash.to_string(),
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: vec!["action.register_agent".to_string()],
            stage: Some(ModuleSubscriptionStage::PreAction),
            filters: None,
        }],
        required_caps: vec![EFFECT_CAP.to_string()],
        artifact_identity: Some(signed_test_artifact_identity(wasm_hash)),
        limits: ModuleLimits::unbounded(),
        abi_contract: ModuleAbiContract::default(),
    }
}

fn route_output() -> ModuleOutput {
    ModuleOutput {
        new_state: Some(vec![0x52, 0x4f, 0x55, 0x54, 0x45]),
        effects: vec![ModuleEffectIntent {
            kind: "http.request".to_string(),
            params: serde_json::json!({"url": "https://example.com/route"}),
            cap_ref: EFFECT_CAP.to_string(),
            cap_slot: None,
        }],
        emits: vec![ModuleEmit {
            kind: "route.module.emit".to_string(),
            payload: serde_json::json!({"route": "late-failure"}),
        }],
        tick_lifecycle: None,
        output_bytes: 32,
    }
}

fn route_action() -> ActionEnvelope {
    ActionEnvelope {
        id: 71,
        action: Action::RegisterAgent {
            agent_id: "route-target".to_string(),
            pos: super::pos(0, 0),
        },
    }
}

fn world_with_route_modules() -> World {
    let wasm_bytes = b"module-routing-transaction";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all(EFFECT_CAP));
    world.set_policy(PolicySet::allow_all());
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .expect("register route artifact");
    activate_module_manifest(&mut world, route_manifest("m.route.a", wasm_hash.as_str()));
    activate_module_manifest(&mut world, route_manifest("m.route.b", wasm_hash.as_str()));
    world
}

fn assert_route_business_unchanged(
    world: &World,
    snapshot_before: &Snapshot,
    root_before: &str,
    pending_before: usize,
    cache_before: usize,
) {
    assert_eq!(world.state(), &snapshot_before.state);
    assert_eq!(
        world.current_state_root_hash().expect("route state root"),
        root_before
    );
    assert_eq!(world.pending_effects_len(), pending_before);
    assert_eq!(world.module_cache_len(), cache_before);
}

#[test]
fn direct_action_route_late_failure_does_not_publish_state_or_effect() {
    let mut world = world_with_route_modules();
    let snapshot_before = world.snapshot();
    let journal_before = world.journal().clone();
    let consensus_before = world.tick_consensus_records().to_vec();
    let root_before = world.current_state_root_hash().expect("initial route root");
    let pending_before = world.pending_effects_len();
    let cache_before = world.module_cache_len();

    let error = world
        .route_action_to_modules(
            &route_action(),
            &mut RouteSandbox::new(route_output(), true),
        )
        .expect_err("late route failure must abort the complete direct route");
    let WorldError::ModuleCallFailed {
        module_id,
        trace_id,
        code,
        detail,
    } = error
    else {
        panic!("expected late module-call failure")
    };
    assert_eq!(module_id, "m.route.b");
    assert_eq!(trace_id, "action-71-m.route.b");
    assert_eq!(code, ModuleCallErrorCode::Trap);
    assert_eq!(detail, "late route failure after first module publication");
    assert_route_business_unchanged(
        &world,
        &snapshot_before,
        root_before.as_str(),
        pending_before,
        cache_before,
    );

    let events = &world.journal().events;
    assert_eq!(events.len(), journal_before.events.len() + 1);
    assert_eq!(
        &events[..journal_before.events.len()],
        journal_before.events.as_slice()
    );
    let Some(WorldEvent {
        id: audit_id,
        body: WorldEventBody::ModuleCallFailed(failure),
        ..
    }) = events.last()
    else {
        panic!("late route failure must retain one audit event")
    };
    assert_eq!(*audit_id, snapshot_before.last_event_id + 1);
    assert_eq!(failure.module_id, "m.route.b");
    assert_eq!(failure.trace_id, "action-71-m.route.b");
    assert_eq!(failure.code, ModuleCallErrorCode::Trap);
    assert_eq!(
        failure.detail,
        "late route failure after first module publication"
    );
    let snapshot_after = world.snapshot();
    assert_eq!(snapshot_after.state, snapshot_before.state);
    assert_eq!(
        snapshot_after.next_intent_id,
        snapshot_before.next_intent_id
    );
    assert_eq!(snapshot_after.intent_id_era, snapshot_before.intent_id_era);
    assert_eq!(
        snapshot_after.pending_effects,
        snapshot_before.pending_effects
    );

    let before_record = consensus_before.last().expect("initial route consensus");
    let after_record = world
        .tick_consensus_records()
        .last()
        .expect("route failure consensus");
    assert_eq!(
        after_record.block.ordered_event_ids.len(),
        before_record.block.ordered_event_ids.len() + 1
    );
    assert_eq!(
        &after_record.block.ordered_event_ids[..before_record.block.ordered_event_ids.len()],
        before_record.block.ordered_event_ids.as_slice()
    );
    assert_eq!(after_record.block.ordered_event_ids.last(), Some(audit_id));
    assert_eq!(after_record.block.header.state_root, root_before);
}

#[test]
fn direct_action_route_infrastructure_failure_is_atomic_and_retry_replays_exactly() {
    let mut world = world_with_route_modules();
    let action = route_action();
    let output = route_output();
    let snapshot_before = world.snapshot();
    let journal_before = world.journal().clone();
    let consensus_before = world.tick_consensus_records().to_vec();
    let root_before = world.current_state_root_hash().expect("initial route root");
    world.fail_next_append_after_publication_prepare_for_test();
    let error = world
        .route_action_to_modules(&action, &mut RouteSandbox::new(output.clone(), false))
        .expect_err("post-prepare route failure must abort before retry");
    assert!(
        matches!(&error, WorldError::ResourceBalanceInvalid { .. }),
        "unexpected infrastructure error: {error:?}"
    );
    assert_eq!(world.snapshot(), snapshot_before);
    assert_eq!(world.journal(), &journal_before);
    assert_eq!(world.tick_consensus_records(), consensus_before.as_slice());
    assert_eq!(
        world.current_state_root_hash().expect("route state root"),
        root_before
    );
    assert!(
        !world
            .journal()
            .events
            .iter()
            .any(|event| matches!(event.body, WorldEventBody::ModuleCallFailed(_)))
    );

    let replay_base = world.snapshot();
    let mut control = world_with_route_modules();
    control
        .route_action_to_modules(&action, &mut RouteSandbox::new(output.clone(), false))
        .expect("control route");
    world
        .route_action_to_modules(&action, &mut RouteSandbox::new(output, false))
        .expect("retry route");
    assert_eq!(world.snapshot(), control.snapshot());
    assert_eq!(world.journal(), control.journal());
    assert_eq!(
        world.tick_consensus_records(),
        control.tick_consensus_records()
    );
    assert_eq!(
        world.runtime_backpressure_stats(),
        control.runtime_backpressure_stats()
    );

    let replayed =
        World::from_snapshot(replay_base, world.journal().clone()).expect("replay direct route");
    assert_eq!(replayed.snapshot(), world.snapshot());
    assert_eq!(replayed.journal(), world.journal());
    assert_eq!(
        replayed.tick_consensus_records(),
        world.tick_consensus_records()
    );
    assert_eq!(
        replayed.runtime_backpressure_stats(),
        world.runtime_backpressure_stats()
    );
}
