use super::pos;
use super::super::*;
use crate::models::BodyKernelView;
use crate::simulator::ResourceKind;

fn install_m1_body_module(world: &mut World) {
    let wasm_bytes = b"m1-body-module";
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .unwrap();

    let module_manifest = ModuleManifest {
        module_id: M1_BODY_MODULE_ID.to_string(),
        name: "M1BodyModule".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Body,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: Vec::new(),
            action_kinds: vec!["action.body_action".to_string()],
            stage: Some(ModuleSubscriptionStage::PreAction),
            filters: None,
        }],
        required_caps: Vec::new(),
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 10,
            max_output_bytes: 4096,
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
fn record_body_attributes_updates_state() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let view = BodyKernelView {
        mass_kg: 120,
        radius_cm: 80,
        thrust_limit: 200,
        cross_section_cm2: 4000,
    };

    world
        .record_body_attributes_update(
            "agent-1",
            view.clone(),
            "boot".to_string(),
            Some(CausedBy::Action(1)),
        )
        .unwrap();

    let agent = world.state().agents.get("agent-1").unwrap();
    assert_eq!(agent.state.body_view, view);

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::BodyAttributesUpdated { agent_id, view, .. }) => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(view.mass_kg, 120);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn record_body_attributes_rejects() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    world
        .record_body_attributes_reject(
            "agent-1",
            "out_of_range".to_string(),
            Some(CausedBy::Action(2)),
        )
        .unwrap();

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::BodyAttributesRejected { agent_id, .. }) => {
            assert_eq!(agent_id, "agent-1");
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn record_body_attributes_update_rejects_out_of_range() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let invalid = BodyKernelView {
        mass_kg: 0,
        radius_cm: 80,
        thrust_limit: 200,
        cross_section_cm2: 4000,
    };

    world
        .record_body_attributes_update(
            "agent-1",
            invalid,
            "boot".to_string(),
            Some(CausedBy::Action(3)),
        )
        .unwrap();

    let agent = world.state().agents.get("agent-1").unwrap();
    assert_eq!(agent.state.body_view, BodyKernelView::default());

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::BodyAttributesRejected { agent_id, reason }) => {
            assert_eq!(agent_id, "agent-1");
            assert!(reason.contains("mass_kg"));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn record_body_attributes_update_rejects_on_rate_violation() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let initial = BodyKernelView {
        mass_kg: 100,
        radius_cm: 80,
        thrust_limit: 200,
        cross_section_cm2: 4000,
    };

    world
        .record_body_attributes_update(
            "agent-1",
            initial.clone(),
            "boot".to_string(),
            Some(CausedBy::Action(4)),
        )
        .unwrap();

    let spike = BodyKernelView {
        mass_kg: 100_000,
        radius_cm: initial.radius_cm,
        thrust_limit: initial.thrust_limit,
        cross_section_cm2: initial.cross_section_cm2,
    };

    world
        .record_body_attributes_update(
            "agent-1",
            spike,
            "upgrade".to_string(),
            Some(CausedBy::Action(5)),
        )
        .unwrap();

    let agent = world.state().agents.get("agent-1").unwrap();
    assert_eq!(agent.state.body_view, initial);

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::BodyAttributesRejected { agent_id, reason }) => {
            assert_eq!(agent_id, "agent-1");
            assert!(reason.contains("rate violation"));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn body_action_updates_view_and_costs_resources() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    install_m1_body_module(&mut world);
    world.set_resource_balance(ResourceKind::Electricity, 100);

    let view = BodyKernelView {
        mass_kg: 120,
        radius_cm: 80,
        thrust_limit: 200,
        cross_section_cm2: 4000,
    };

    world.submit_action(Action::BodyAction {
        agent_id: "agent-1".to_string(),
        kind: "boot".to_string(),
        payload: serde_json::to_value(view.clone()).unwrap(),
    });

    let mut sandbox = BuiltinModuleSandbox::new().register_builtin(
        M1_BODY_MODULE_ID,
        M1BodyModule::default(),
    );
    world.step_with_modules(&mut sandbox).unwrap();

    let agent = world.state().agents.get("agent-1").unwrap();
    assert_eq!(agent.state.body_view, view);
    assert_eq!(
        world.resource_balance(ResourceKind::Electricity),
        100 - M1_BODY_ACTION_COST_ELECTRICITY
    );
}

#[test]
fn body_action_rejects_when_insufficient_resources() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    install_m1_body_module(&mut world);
    world.set_resource_balance(ResourceKind::Electricity, 0);

    let view = BodyKernelView {
        mass_kg: 120,
        radius_cm: 80,
        thrust_limit: 200,
        cross_section_cm2: 4000,
    };

    world.submit_action(Action::BodyAction {
        agent_id: "agent-1".to_string(),
        kind: "boot".to_string(),
        payload: serde_json::to_value(view).unwrap(),
    });

    let mut sandbox = BuiltinModuleSandbox::new().register_builtin(
        M1_BODY_MODULE_ID,
        M1BodyModule::default(),
    );
    world.step_with_modules(&mut sandbox).unwrap();

    let last = world.journal().events.last().unwrap();
    match &last.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => {
            match reason {
                RejectReason::InsufficientResources { .. } => {}
                other => panic!("unexpected reject reason: {other:?}"),
            }
        }
        other => panic!("unexpected event: {other:?}"),
    }
}
