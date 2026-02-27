use super::pos;
use crate::runtime::{Action, DomainEvent, MaterialLedgerId, World, WorldEventBody};
use crate::simulator::ResourceKind;
use agent_world_wasm_abi::{FactoryModuleSpec, MaterialStack, RecipeExecutionPlan};

fn factory_spec(factory_id: &str, build_time_ticks: u32, recipe_slots: u16) -> FactoryModuleSpec {
    FactoryModuleSpec {
        factory_id: factory_id.to_string(),
        display_name: "Test Factory".to_string(),
        tier: 1,
        tags: vec!["assembly".to_string()],
        build_cost: vec![
            MaterialStack::new("steel_plate", 10),
            MaterialStack::new("circuit_board", 2),
        ],
        build_time_ticks,
        base_power_draw: 5,
        recipe_slots,
        throughput_bps: 10_000,
        maintenance_per_tick: 1,
    }
}

#[test]
fn due_recipe_jobs_prioritize_survival_over_expansion() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "builder-a".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");

    world
        .set_material_balance("steel_plate", 20)
        .expect("seed build steel");
    world
        .set_material_balance("circuit_board", 4)
        .expect("seed build circuits");
    world.submit_action(Action::BuildFactory {
        builder_agent_id: "builder-a".to_string(),
        site_id: "site-1".to_string(),
        spec: factory_spec("factory.priority", 1, 2),
    });
    world.step().expect("start factory build");
    world.step().expect("factory ready");

    world
        .set_material_balance("iron_ingot", 4)
        .expect("seed recipe input");
    world.set_resource_balance(ResourceKind::Electricity, 20);

    let expansion_plan = RecipeExecutionPlan::accepted(
        1,
        vec![MaterialStack::new("iron_ingot", 2)],
        vec![MaterialStack::new("outpost_kit", 1)],
        Vec::new(),
        2,
        1,
    );
    let survival_plan = RecipeExecutionPlan::accepted(
        1,
        vec![MaterialStack::new("iron_ingot", 2)],
        vec![MaterialStack::new("oxygen_pack", 1)],
        Vec::new(),
        2,
        1,
    );

    // Submit expansion first to verify due-job completion still prioritizes survival.
    world.submit_action(Action::ScheduleRecipe {
        requester_agent_id: "builder-a".to_string(),
        factory_id: "factory.priority".to_string(),
        recipe_id: "recipe.expand.outpost".to_string(),
        plan: expansion_plan,
    });
    world.submit_action(Action::ScheduleRecipe {
        requester_agent_id: "builder-a".to_string(),
        factory_id: "factory.priority".to_string(),
        recipe_id: "recipe.survival.oxygen".to_string(),
        plan: survival_plan,
    });
    world.step().expect("start recipes");
    assert_eq!(world.pending_recipe_jobs_len(), 2);

    let before = world.journal().events.len();
    world.step().expect("complete recipes");

    let completed_recipe_ids: Vec<String> = world.journal().events[before..]
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::Domain(DomainEvent::RecipeCompleted { recipe_id, .. }) => {
                Some(recipe_id.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        completed_recipe_ids,
        vec![
            "recipe.survival.oxygen".to_string(),
            "recipe.expand.outpost".to_string(),
        ]
    );
}

#[test]
fn logistics_sla_metrics_are_observable_after_transit_completion() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "operator-a".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register operator");

    world
        .set_ledger_material_balance(MaterialLedgerId::site("site-a"), "copper_wire", 100)
        .expect("seed source");
    world.submit_action(Action::TransferMaterial {
        requester_agent_id: "operator-a".to_string(),
        from_ledger: MaterialLedgerId::site("site-a"),
        to_ledger: MaterialLedgerId::site("site-b"),
        kind: "copper_wire".to_string(),
        amount: 50,
        distance_km: 100,
    });
    world.step().expect("start transit");
    world.step().expect("complete transit");

    let metrics = world.logistics_sla_metrics();
    assert_eq!(metrics.completed_transits, 1);
    assert_eq!(metrics.fulfilled_transits, 1);
    assert_eq!(metrics.breached_transits, 0);
    assert_eq!(metrics.total_delay_ticks, 0);
    assert_eq!(metrics.breach_rate(), 0.0);
    assert_eq!(metrics.fulfillment_rate(), 1.0);
}
