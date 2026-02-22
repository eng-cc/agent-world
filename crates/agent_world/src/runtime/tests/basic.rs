use super::super::*;
use super::pos;
use crate::simulator::ResourceKind;

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
        WorldEventBody::Domain(DomainEvent::ActionRejected {
            action_id: id,
            reason,
        }) => {
            assert_eq!(*id, action_id);
            assert!(matches!(reason, RejectReason::AgentNotFound { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
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

#[test]
fn new_world_migrates_legacy_world_materials_into_material_ledgers() {
    let mut state = WorldState::default();
    state.material_ledgers.clear();
    state.materials.insert("iron_ingot".to_string(), 7);

    let world = World::new_with_state(state);

    assert_eq!(
        world.ledger_material_balance(&MaterialLedgerId::world(), "iron_ingot"),
        7
    );
    assert_eq!(world.material_balance("iron_ingot"), 7);
}

#[test]
fn action_id_rolls_over_into_next_era() {
    let mut world = World::new();
    let mut snapshot = world.snapshot();
    snapshot.next_action_id = u64::MAX;
    snapshot.action_id_era = 7;

    world = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");

    let first_id = world.submit_action(Action::RegisterAgent {
        agent_id: "agent-max".to_string(),
        pos: pos(0.0, 0.0),
    });
    let second_id = world.submit_action(Action::RegisterAgent {
        agent_id: "agent-wrap".to_string(),
        pos: pos(1.0, 1.0),
    });

    assert_eq!(first_id, u64::MAX);
    assert_eq!(second_id, 1);
    let rolled = world.snapshot();
    assert_eq!(rolled.action_id_era, 8);
    assert_eq!(rolled.next_action_id, 2);
}

#[test]
fn event_id_rolls_over_into_next_era() {
    let mut world = World::new();
    let mut snapshot = world.snapshot();
    snapshot.last_event_id = u64::MAX;
    snapshot.event_id_era = 3;

    world = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-max".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("step 1");
    assert_eq!(world.journal().events.last().expect("event").id, u64::MAX);

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-wrap".to_string(),
        pos: pos(1.0, 1.0),
    });
    world.step().expect("step 2");
    assert_eq!(world.journal().events.last().expect("event").id, 1);

    let rolled = world.snapshot();
    assert_eq!(rolled.event_id_era, 4);
    assert_eq!(rolled.last_event_id, 1);
}

#[test]
fn adjust_resource_balance_saturates_on_overflow() {
    let mut world = World::new();
    world.set_resource_balance(ResourceKind::Data, i64::MAX - 1);
    let next = world.adjust_resource_balance(ResourceKind::Data, 9);

    assert_eq!(next, i64::MAX);
    assert_eq!(world.resource_balance(ResourceKind::Data), i64::MAX);
}
