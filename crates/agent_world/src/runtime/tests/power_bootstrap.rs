#![cfg(feature = "wasmtime")]

use super::super::*;
use super::pos;

const M1_BUILTIN_WASM_ARTIFACT: &[u8] =
    include_bytes!("../world/artifacts/m1_builtin_modules.wasm");
const M1_BUILTIN_WASM_ARTIFACT_SHA256: &str =
    include_str!("../world/artifacts/m1_builtin_modules.wasm.sha256");

fn has_active(world: &World, module_id: &str) -> bool {
    world.module_registry().active.contains_key(module_id)
}

fn power_module_sandbox() -> WasmExecutor {
    WasmExecutor::new(WasmExecutorConfig::default())
}

fn apply_module_changes(world: &mut World, actor: &str, changes: ModuleChangeSet) {
    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).expect("serialize module change set"),
    );
    let manifest = Manifest {
        version: world.manifest().version.saturating_add(1),
        content: serde_json::Value::Object(content),
    };

    let proposal_id = world
        .propose_manifest_update(manifest, actor.to_string())
        .expect("propose changes");
    world.shadow_proposal(proposal_id).expect("shadow proposal");
    world
        .approve_proposal(proposal_id, actor.to_string(), ProposalDecision::Approve)
        .expect("approve proposal");
    world.apply_proposal(proposal_id).expect("apply proposal");
}

#[test]
fn embedded_m1_builtin_wasm_hash_manifest_matches_artifact() {
    let expected = M1_BUILTIN_WASM_ARTIFACT_SHA256.trim();
    assert_eq!(expected.len(), 64);
    assert!(expected.chars().all(|ch| ch.is_ascii_hexdigit()));

    let actual = util::sha256_hex(M1_BUILTIN_WASM_ARTIFACT);
    assert_eq!(actual, expected);
}

#[test]
fn install_power_bootstrap_modules_registers_and_activates() {
    let mut world = World::new();
    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("install modules");

    assert!(has_active(&world, M1_RADIATION_POWER_MODULE_ID));
    assert!(has_active(&world, M1_STORAGE_POWER_MODULE_ID));

    let radiation_key =
        ModuleRegistry::record_key(M1_RADIATION_POWER_MODULE_ID, M1_POWER_MODULE_VERSION);
    let storage_key =
        ModuleRegistry::record_key(M1_STORAGE_POWER_MODULE_ID, M1_POWER_MODULE_VERSION);
    assert!(world.module_registry().records.contains_key(&radiation_key));
    assert!(world.module_registry().records.contains_key(&storage_key));
}

#[test]
fn install_agent_default_modules_registers_and_activates() {
    let mut world = World::new();
    world
        .install_m1_agent_default_modules("bootstrap")
        .expect("install default modules");

    assert!(has_active(&world, M1_SENSOR_MODULE_ID));
    assert!(has_active(&world, M1_MOBILITY_MODULE_ID));
    assert!(has_active(&world, M1_MEMORY_MODULE_ID));
    assert!(has_active(&world, M1_STORAGE_CARGO_MODULE_ID));

    let sensor_key =
        ModuleRegistry::record_key(M1_SENSOR_MODULE_ID, M1_AGENT_DEFAULT_MODULE_VERSION);
    let mobility_key =
        ModuleRegistry::record_key(M1_MOBILITY_MODULE_ID, M1_AGENT_DEFAULT_MODULE_VERSION);
    let memory_key =
        ModuleRegistry::record_key(M1_MEMORY_MODULE_ID, M1_AGENT_DEFAULT_MODULE_VERSION);
    let cargo_key =
        ModuleRegistry::record_key(M1_STORAGE_CARGO_MODULE_ID, M1_AGENT_DEFAULT_MODULE_VERSION);

    assert!(world.module_registry().records.contains_key(&sensor_key));
    assert!(world.module_registry().records.contains_key(&mobility_key));
    assert!(world.module_registry().records.contains_key(&memory_key));
    assert!(world.module_registry().records.contains_key(&cargo_key));
}

#[test]
fn install_agent_default_modules_is_idempotent() {
    let mut world = World::new();
    world
        .install_m1_agent_default_modules("bootstrap")
        .expect("first install");
    let event_len = world.journal().len();

    world
        .install_m1_agent_default_modules("bootstrap")
        .expect("second install");

    assert_eq!(world.journal().len(), event_len);
}

#[test]
fn install_scenario_bootstrap_modules_supports_default_package_toggle() {
    let mut world = World::new();
    world
        .install_m1_scenario_bootstrap_modules(
            "bootstrap",
            M1ScenarioBootstrapConfig {
                install_default_module_package: false,
            },
        )
        .expect("install scenario bootstrap modules");

    assert!(has_active(&world, M1_RADIATION_POWER_MODULE_ID));
    assert!(has_active(&world, M1_STORAGE_POWER_MODULE_ID));
    assert!(!has_active(&world, M1_SENSOR_MODULE_ID));
    assert!(!has_active(&world, M1_MOBILITY_MODULE_ID));
    assert!(!has_active(&world, M1_MEMORY_MODULE_ID));
    assert!(!has_active(&world, M1_STORAGE_CARGO_MODULE_ID));
}

#[test]
fn install_scenario_bootstrap_modules_is_idempotent() {
    let mut world = World::new();
    let config = M1ScenarioBootstrapConfig::default();

    world
        .install_m1_scenario_bootstrap_modules("bootstrap", config)
        .expect("first scenario install");
    assert!(has_active(&world, M1_RADIATION_POWER_MODULE_ID));
    assert!(has_active(&world, M1_STORAGE_POWER_MODULE_ID));
    assert!(has_active(&world, M1_SENSOR_MODULE_ID));
    assert!(has_active(&world, M1_MOBILITY_MODULE_ID));
    assert!(has_active(&world, M1_MEMORY_MODULE_ID));
    assert!(has_active(&world, M1_STORAGE_CARGO_MODULE_ID));
    let event_len = world.journal().len();

    world
        .install_m1_scenario_bootstrap_modules("bootstrap", config)
        .expect("second scenario install");

    assert_eq!(world.journal().len(), event_len);
}

#[test]
fn install_power_bootstrap_modules_is_idempotent() {
    let mut world = World::new();
    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("first install");
    let event_len = world.journal().len();

    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("second install");

    assert_eq!(world.journal().len(), event_len);
}

#[test]
fn install_power_bootstrap_modules_reactivates_registered_version() {
    let mut world = World::new();
    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("initial install");

    let registered_count = world.module_registry().records.len();
    apply_module_changes(
        &mut world,
        "bootstrap",
        ModuleChangeSet {
            deactivate: vec![ModuleDeactivation {
                module_id: M1_STORAGE_POWER_MODULE_ID.to_string(),
                reason: "bootstrap test deactivate".to_string(),
            }],
            ..ModuleChangeSet::default()
        },
    );
    assert!(!has_active(&world, M1_STORAGE_POWER_MODULE_ID));

    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("reactivate install");

    assert!(has_active(&world, M1_STORAGE_POWER_MODULE_ID));
    assert_eq!(world.module_registry().records.len(), registered_count);
}

#[test]
fn radiation_module_emits_harvest_event() {
    let mut world = World::new();
    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("install modules");

    let mut sandbox = power_module_sandbox();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register step");

    world.submit_action(Action::QueryObservation {
        agent_id: "agent-1".to_string(),
    });
    world.step_with_modules(&mut sandbox).expect("tick step");

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(100_000.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("harvest from move step");

    let found = world.journal().events.iter().any(|event| {
        matches!(
            &event.body,
            WorldEventBody::ModuleEmitted(module_event)
                if module_event.module_id == M1_RADIATION_POWER_MODULE_ID
                    && module_event.kind == "power.radiation_harvest"
        )
    });
    assert!(found);
}

#[test]
fn storage_module_blocks_continuous_move_when_power_runs_out() {
    let mut world = World::new();
    world
        .install_m1_power_bootstrap_modules("bootstrap")
        .expect("install modules");

    let mut sandbox = power_module_sandbox();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register step");

    for idx in 0..5 {
        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: pos((idx as f64 + 1.0) * 100_000.0, 0.0),
        });
        world
            .step_with_modules(&mut sandbox)
            .expect("move evaluation step");
    }

    let denied = world.journal().events.iter().any(|event| {
        matches!(
            &event.body,
            WorldEventBody::Domain(DomainEvent::ActionRejected {
                reason: RejectReason::RuleDenied { notes },
                ..
            }) if notes.iter().any(|note| note.contains("storage insufficient"))
        )
    });
    let rejections: Vec<String> = world
        .journal()
        .events
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => {
                Some(format!("{reason:?}"))
            }
            _ => None,
        })
        .collect();
    assert!(
        denied,
        "expected storage deny, got rejections: {}",
        rejections.join(" | ")
    );
}
