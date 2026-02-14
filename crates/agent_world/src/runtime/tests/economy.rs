use super::pos;
use crate::runtime::{Action, DomainEvent, RejectReason, World, WorldEventBody};
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
fn build_factory_consumes_materials_and_completes_after_delay() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "builder-a".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");

    world
        .set_material_balance("steel_plate", 20)
        .expect("seed steel");
    world
        .set_material_balance("circuit_board", 4)
        .expect("seed circuits");

    world.submit_action(Action::BuildFactory {
        builder_agent_id: "builder-a".to_string(),
        site_id: "site-1".to_string(),
        spec: factory_spec("factory.alpha", 2, 1),
    });

    world.step().expect("start factory build");
    assert_eq!(world.pending_factory_builds_len(), 1);
    assert!(!world.has_factory("factory.alpha"));
    assert_eq!(world.material_balance("steel_plate"), 10);
    assert_eq!(world.material_balance("circuit_board"), 2);

    let started = world
        .journal()
        .events
        .last()
        .expect("factory build started event");
    match &started.body {
        WorldEventBody::Domain(DomainEvent::FactoryBuildStarted { spec, .. }) => {
            assert_eq!(spec.factory_id, "factory.alpha")
        }
        other => panic!("expected FactoryBuildStarted, got {other:?}"),
    }

    world.step().expect("tick without completion");
    assert_eq!(world.pending_factory_builds_len(), 1);
    assert!(!world.has_factory("factory.alpha"));

    world.step().expect("complete factory build");
    assert_eq!(world.pending_factory_builds_len(), 0);
    assert!(world.has_factory("factory.alpha"));

    let completed = world.journal().events.last().expect("factory built event");
    match &completed.body {
        WorldEventBody::Domain(DomainEvent::FactoryBuilt { spec, .. }) => {
            assert_eq!(spec.factory_id, "factory.alpha")
        }
        other => panic!("expected FactoryBuilt, got {other:?}"),
    }
}

#[test]
fn schedule_recipe_consumes_inputs_and_power_then_produces_outputs() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "builder-a".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");

    world
        .set_material_balance("steel_plate", 11)
        .expect("seed build steel");
    world
        .set_material_balance("circuit_board", 2)
        .expect("seed build circuits");

    world.submit_action(Action::BuildFactory {
        builder_agent_id: "builder-a".to_string(),
        site_id: "site-1".to_string(),
        spec: factory_spec("factory.recipe", 1, 1),
    });
    world.step().expect("start factory build");
    world.step().expect("factory ready");
    assert!(world.has_factory("factory.recipe"));

    world
        .set_material_balance("iron_ingot", 6)
        .expect("seed recipe input");
    world.set_resource_balance(ResourceKind::Electricity, 20);

    let plan = RecipeExecutionPlan::accepted(
        2,
        vec![MaterialStack::new("iron_ingot", 6)],
        vec![MaterialStack::new("motor_mk1", 2)],
        vec![MaterialStack::new("metal_scrap", 1)],
        7,
        1,
    );

    world.submit_action(Action::ScheduleRecipe {
        requester_agent_id: "builder-a".to_string(),
        factory_id: "factory.recipe".to_string(),
        recipe_id: "recipe.motor.mk1".to_string(),
        plan,
    });

    world.step().expect("start recipe");
    assert_eq!(world.pending_recipe_jobs_len(), 1);
    assert_eq!(world.material_balance("iron_ingot"), 0);
    assert_eq!(world.resource_balance(ResourceKind::Electricity), 13);

    let started = world.journal().events.last().expect("recipe started event");
    match &started.body {
        WorldEventBody::Domain(DomainEvent::RecipeStarted { recipe_id, .. }) => {
            assert_eq!(recipe_id, "recipe.motor.mk1")
        }
        other => panic!("expected RecipeStarted, got {other:?}"),
    }

    world.step().expect("complete recipe");
    assert_eq!(world.pending_recipe_jobs_len(), 0);
    assert_eq!(world.material_balance("motor_mk1"), 2);
    assert_eq!(world.material_balance("metal_scrap"), 1);

    let completed = world
        .journal()
        .events
        .last()
        .expect("recipe completed event");
    match &completed.body {
        WorldEventBody::Domain(DomainEvent::RecipeCompleted { recipe_id, .. }) => {
            assert_eq!(recipe_id, "recipe.motor.mk1")
        }
        other => panic!("expected RecipeCompleted, got {other:?}"),
    }
}

#[test]
fn schedule_recipe_rejects_when_factory_slots_are_full() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "builder-a".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");

    world
        .set_material_balance("steel_plate", 10)
        .expect("seed build steel");
    world
        .set_material_balance("circuit_board", 2)
        .expect("seed build circuits");

    world.submit_action(Action::BuildFactory {
        builder_agent_id: "builder-a".to_string(),
        site_id: "site-1".to_string(),
        spec: factory_spec("factory.slot", 1, 1),
    });
    world.step().expect("start factory build");
    world.step().expect("factory ready");

    world
        .set_material_balance("gear", 8)
        .expect("seed recipe input");
    world.set_resource_balance(ResourceKind::Electricity, 50);

    let plan_a = RecipeExecutionPlan::accepted(
        1,
        vec![MaterialStack::new("gear", 2)],
        vec![MaterialStack::new("module_a", 1)],
        Vec::new(),
        2,
        3,
    );
    world.submit_action(Action::ScheduleRecipe {
        requester_agent_id: "builder-a".to_string(),
        factory_id: "factory.slot".to_string(),
        recipe_id: "recipe.a".to_string(),
        plan: plan_a,
    });
    world.step().expect("start recipe a");
    assert_eq!(world.pending_recipe_jobs_len(), 1);

    let plan_b = RecipeExecutionPlan::accepted(
        1,
        vec![MaterialStack::new("gear", 2)],
        vec![MaterialStack::new("module_b", 1)],
        Vec::new(),
        2,
        1,
    );
    world.submit_action(Action::ScheduleRecipe {
        requester_agent_id: "builder-a".to_string(),
        factory_id: "factory.slot".to_string(),
        recipe_id: "recipe.b".to_string(),
        plan: plan_b,
    });
    world.step().expect("reject recipe b");

    let rejected = world.journal().events.last().expect("rejection event");
    match &rejected.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => match reason {
            RejectReason::FactoryBusy {
                factory_id,
                active_jobs,
                recipe_slots,
            } => {
                assert_eq!(factory_id, "factory.slot");
                assert_eq!(*active_jobs, 1);
                assert_eq!(*recipe_slots, 1);
            }
            other => panic!("expected FactoryBusy reject reason, got {other:?}"),
        },
        other => panic!("expected ActionRejected, got {other:?}"),
    }
}

#[test]
fn build_factory_rejects_when_materials_insufficient() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "builder-a".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");

    world
        .set_material_balance("steel_plate", 3)
        .expect("seed limited steel");
    world
        .set_material_balance("circuit_board", 2)
        .expect("seed circuits");

    world.submit_action(Action::BuildFactory {
        builder_agent_id: "builder-a".to_string(),
        site_id: "site-1".to_string(),
        spec: factory_spec("factory.fail", 1, 1),
    });
    world.step().expect("build rejected");

    let rejected = world.journal().events.last().expect("rejection event");
    match &rejected.body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => match reason {
            RejectReason::InsufficientMaterial {
                material_kind,
                requested,
                available,
            } => {
                assert_eq!(material_kind, "steel_plate");
                assert_eq!(*requested, 10);
                assert_eq!(*available, 3);
            }
            other => panic!("expected InsufficientMaterial reject reason, got {other:?}"),
        },
        other => panic!("expected ActionRejected, got {other:?}"),
    }
}
