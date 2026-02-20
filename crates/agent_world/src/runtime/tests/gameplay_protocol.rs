use super::super::*;
use super::pos;

fn register_agents(world: &mut World, agent_ids: &[&str]) {
    for (index, agent_id) in agent_ids.iter().enumerate() {
        world.submit_action(Action::RegisterAgent {
            agent_id: (*agent_id).to_string(),
            pos: pos(index as f64, 0.0),
        });
    }
    world.step().expect("register agents");
}

fn last_domain_event(world: &World) -> &DomainEvent {
    let event = world.journal().events.last().expect("domain event");
    let WorldEventBody::Domain(domain_event) = &event.body else {
        panic!("expected domain event");
    };
    domain_event
}

#[test]
fn gameplay_protocol_actions_drive_persisted_state() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c", "d"]);

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "a".to_string(),
        alliance_id: "alliance.red".to_string(),
        members: vec!["b".to_string()],
        charter: "mutual defense".to_string(),
    });
    world.step().expect("form red alliance");

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "c".to_string(),
        alliance_id: "alliance.blue".to_string(),
        members: vec!["d".to_string()],
        charter: "logistics pact".to_string(),
    });
    world.step().expect("form blue alliance");

    world.submit_action(Action::DeclareWar {
        initiator_agent_id: "a".to_string(),
        war_id: "war.001".to_string(),
        aggressor_alliance_id: "alliance.red".to_string(),
        defender_alliance_id: "alliance.blue".to_string(),
        objective: "control asteroid belt".to_string(),
        intensity: 2,
    });
    world.step().expect("declare war");

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.energy_tax".to_string(),
        option: "approve".to_string(),
        weight: 3,
    });
    world.step().expect("cast governance vote");

    world.submit_action(Action::ResolveCrisis {
        resolver_agent_id: "c".to_string(),
        crisis_id: "crisis.solar_storm.1".to_string(),
        strategy: "redistribute shield grid".to_string(),
        success: true,
    });
    world.step().expect("resolve crisis");

    world.submit_action(Action::GrantMetaProgress {
        operator_agent_id: "a".to_string(),
        target_agent_id: "b".to_string(),
        track: "campaign".to_string(),
        points: 15,
        achievement_id: Some("first_alliance_win".to_string()),
    });
    world.step().expect("grant meta progress");

    let red = world
        .state()
        .alliances
        .get("alliance.red")
        .expect("red alliance");
    assert_eq!(red.members, vec!["a".to_string(), "b".to_string()]);

    let war = world.state().wars.get("war.001").expect("war record");
    assert_eq!(war.aggressor_alliance_id, "alliance.red");
    assert_eq!(war.defender_alliance_id, "alliance.blue");
    assert!(war.active);

    let governance = world
        .state()
        .governance_votes
        .get("proposal.energy_tax")
        .expect("governance vote state");
    assert_eq!(governance.total_weight, 3);
    assert_eq!(governance.tallies.get("approve"), Some(&3_u64));

    let crisis = world
        .state()
        .crises
        .get("crisis.solar_storm.1")
        .expect("crisis state");
    assert!(crisis.success);
    assert_eq!(crisis.impact, 10);

    let progress = world.state().meta_progress.get("b").expect("meta progress");
    assert_eq!(progress.total_points, 15);
    assert_eq!(progress.track_points.get("campaign"), Some(&15));
    assert_eq!(
        progress.achievements,
        vec!["first_alliance_win".to_string()]
    );

    assert!(matches!(
        last_domain_event(&world),
        DomainEvent::MetaProgressGranted { .. }
    ));
}

#[test]
fn declare_war_rejects_initiator_outside_aggressor_alliance() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c", "d"]);

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "a".to_string(),
        alliance_id: "alliance.red".to_string(),
        members: vec!["b".to_string()],
        charter: "charter.red".to_string(),
    });
    world.step().expect("form red alliance");

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "c".to_string(),
        alliance_id: "alliance.blue".to_string(),
        members: vec!["d".to_string()],
        charter: "charter.blue".to_string(),
    });
    world.step().expect("form blue alliance");

    world.submit_action(Action::DeclareWar {
        initiator_agent_id: "c".to_string(),
        war_id: "war.invalid".to_string(),
        aggressor_alliance_id: "alliance.red".to_string(),
        defender_alliance_id: "alliance.blue".to_string(),
        objective: "invalid".to_string(),
        intensity: 1,
    });
    world.step().expect("reject invalid war declare");

    match last_domain_event(&world) {
        DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        } => {
            assert!(notes
                .iter()
                .any(|note| note.contains("is not a member of aggressor alliance")));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn governance_vote_recast_replaces_previous_tally() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.runtime".to_string(),
        option: "approve".to_string(),
        weight: 3,
    });
    world.step().expect("first vote");

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.runtime".to_string(),
        option: "reject".to_string(),
        weight: 1,
    });
    world.step().expect("recast vote");

    let governance = world
        .state()
        .governance_votes
        .get("proposal.runtime")
        .expect("governance vote state");
    assert_eq!(governance.total_weight, 1);
    assert_eq!(governance.tallies.get("reject"), Some(&1_u64));
    assert!(!governance.tallies.contains_key("approve"));
    assert_eq!(governance.votes_by_agent.len(), 1);
}
