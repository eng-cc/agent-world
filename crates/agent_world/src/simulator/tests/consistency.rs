use super::*;

fn run_action_sequence(kernel: &mut WorldKernel, actions: &[Action]) {
    for action in actions {
        kernel.submit_action(action.clone());
        kernel.step().expect("action event");
    }
}

#[test]
fn replay_from_snapshot_matches_same_seed_and_action_sequence() {
    let mut config = WorldConfig::default();
    config.economy.refine_electricity_cost_per_kg = 1;
    config.physics.heat_factor = 0;

    let mut init = WorldInitConfig::default();
    init.seed = 123;
    init.agents.count = 1;
    init.asteroid_fragment.enabled = false;

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("initialize kernel");

    let actions = vec![
        Action::HarvestRadiation {
            agent_id: "agent-0".to_string(),
            max_amount: 10,
        },
        Action::HarvestRadiation {
            agent_id: "agent-0".to_string(),
            max_amount: 10,
        },
        Action::RefineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-0".to_string(),
            },
            compound_mass_g: 1_000,
        },
    ];

    run_action_sequence(&mut kernel, &actions[..1]);
    let mid_snapshot = kernel.snapshot();

    run_action_sequence(&mut kernel, &actions[1..]);

    let journal = kernel.journal_snapshot();
    let replayed = WorldKernel::replay_from_snapshot(mid_snapshot, journal.clone())
        .expect("replay from snapshot");
    assert_eq!(replayed.model(), kernel.model());

    let restored =
        WorldKernel::from_snapshot(kernel.snapshot(), journal).expect("restore snapshot");
    assert_eq!(restored.model(), kernel.model());
}
