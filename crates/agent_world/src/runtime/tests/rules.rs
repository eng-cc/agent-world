use super::pos;
use super::super::*;
use crate::simulator::ResourceKind;

fn install_m1_move_rule(world: &mut World) {
    let wasm_bytes = b"m1-move-rule";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: M1_MOVE_RULE_MODULE_ID.to_string(),
        name: "M1MoveRule".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Rule,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![
            ModuleSubscription {
                event_kinds: vec![
                    "domain.agent_registered".to_string(),
                    "domain.agent_moved".to_string(),
                ],
                action_kinds: Vec::new(),
                stage: Some(ModuleSubscriptionStage::PostEvent),
                filters: None,
            },
            ModuleSubscription {
                event_kinds: Vec::new(),
                action_kinds: vec!["action.move_agent".to_string()],
                stage: Some(ModuleSubscriptionStage::PreAction),
                filters: None,
            },
        ],
        required_caps: Vec::new(),
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 10,
            max_output_bytes: 2048,
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
}

#[test]
fn rule_decision_override_and_cost_apply() {
    let mut world = World::new();
    world.set_policy(PolicySet::allow_all());
    world.set_resource_balance(ResourceKind::Electricity, 5);

    let wasm_bytes = b"rule-decision-override";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.rule".to_string(),
        name: "Rule".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Pure,
        role: ModuleRole::Rule,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["call".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: vec!["action.register_agent".to_string()],
            stage: Some(ModuleSubscriptionStage::PreAction),
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

    let override_action = Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(2.0, 3.0),
    };
    let mut cost = ResourceDelta::default();
    cost.entries.insert(ResourceKind::Electricity, -3);
    let decision = RuleDecision {
        action_id: 1,
        verdict: RuleVerdict::Modify,
        override_action: Some(override_action.clone()),
        cost,
        notes: vec!["override".to_string()],
    };
    let output = ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: "rule.decision".to_string(),
            payload: serde_json::to_value(&decision).unwrap(),
        }],
        output_bytes: 128,
    };
    let mut sandbox = FixedSandbox::succeed(output);

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).unwrap();

    let agent = world.state().agents.get("agent-1").unwrap();
    assert_eq!(agent.state.pos, pos(2.0, 3.0));
    assert_eq!(world.resource_balance(ResourceKind::Electricity), 2);
    assert!(world
        .journal()
        .events
        .iter()
        .any(|event| matches!(event.body, WorldEventBody::ActionOverridden(_))));
}

#[test]
fn rule_decision_rejects_on_insufficient_resources() {
    let mut world = World::new();
    world.set_policy(PolicySet::allow_all());
    world.set_resource_balance(ResourceKind::Electricity, 1);

    let wasm_bytes = b"rule-decision-cost";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: "m.rule.cost".to_string(),
        name: "RuleCost".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Pure,
        role: ModuleRole::Rule,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["call".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: vec!["action.register_agent".to_string()],
            stage: Some(ModuleSubscriptionStage::PreAction),
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

    let mut cost = ResourceDelta::default();
    cost.entries.insert(ResourceKind::Electricity, -3);
    let decision = RuleDecision {
        action_id: 1,
        verdict: RuleVerdict::Allow,
        override_action: None,
        cost,
        notes: Vec::new(),
    };
    let output = ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: "rule.decision".to_string(),
            payload: serde_json::to_value(&decision).unwrap(),
        }],
        output_bytes: 128,
    };
    let mut sandbox = FixedSandbox::succeed(output);

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).unwrap();

    assert!(world.state().agents.get("agent-1").is_none());
    assert_eq!(world.resource_balance(ResourceKind::Electricity), 1);
    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => {
            assert!(matches!(reason, RejectReason::InsufficientResources { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn m1_move_rule_rejects_when_insufficient_resources() {
    let mut world = World::new();
    world.set_resource_balance(ResourceKind::Electricity, 0);
    install_m1_move_rule(&mut world);

    let mut sandbox = BuiltinModuleSandbox::new()
        .register_builtin(M1_MOVE_RULE_MODULE_ID, M1MoveRuleModule::default());

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).unwrap();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(100_000.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).unwrap();

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => {
            assert!(matches!(reason, RejectReason::InsufficientResources { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
    assert_eq!(world.resource_balance(ResourceKind::Electricity), 0);
}

#[test]
fn m1_move_rule_denies_same_position() {
    let mut world = World::new();
    world.set_resource_balance(ResourceKind::Electricity, 10);
    install_m1_move_rule(&mut world);

    let mut sandbox = BuiltinModuleSandbox::new()
        .register_builtin(M1_MOVE_RULE_MODULE_ID, M1MoveRuleModule::default());

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).unwrap();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(0.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).unwrap();

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => {
            assert!(matches!(reason, RejectReason::RuleDenied { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
    assert_eq!(world.resource_balance(ResourceKind::Electricity), 10);
}
