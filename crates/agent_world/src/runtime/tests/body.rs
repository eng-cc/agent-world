use super::pos;
use super::super::*;
use crate::models::BodyKernelView;

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
