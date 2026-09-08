use super::pos;
use crate::runtime::state::industry_transition::{
    PreparedFactoryLifecycle, PreparedRecipeLifecycle,
};
use crate::runtime::*;
use crate::simulator::ResourceKind;
use oasis7_wasm_abi::{FactoryModuleSpec, MaterialStack};

fn register(world: &mut World) {
    world.submit_action(Action::RegisterAgent {
        agent_id: "actor".into(),
        pos: pos(0, 0),
    });
    world.step().unwrap();
}

fn spec() -> FactoryModuleSpec {
    FactoryModuleSpec {
        factory_id: "factory".into(),
        display_name: "Factory".into(),
        tier: 1,
        tags: vec!["assembly".into()],
        build_cost: vec![MaterialStack::new("steel", 2)],
        build_time_ticks: 1,
        base_power_draw: 1,
        recipe_slots: 1,
        throughput_bps: 10_000,
        maintenance_per_tick: 1,
    }
}

#[test]
fn factory_build_started_cost_preflight_reads_legacy_world_materials() {
    let mut world = World::new();
    register(&mut world);
    let mut state = world.state().clone();
    state.materials.insert("steel".into(), 10);
    state.material_ledgers.remove(&MaterialLedgerId::world());
    let event = DomainEvent::FactoryBuildStarted {
        job_id: 7001,
        builder_agent_id: "actor".into(),
        site_id: "legacy-site".into(),
        spec: spec(),
        consume_ledger: MaterialLedgerId::world(),
        ready_at: state.time + 1,
        contract_version: Some(0),
        site_authority_revision: None,
        site_location_id: None,
        location_anchor_revision: None,
        construction_power_obligation: None,
    };
    let before = state.clone();
    let mut insufficient = event.clone();
    if let DomainEvent::FactoryBuildStarted { spec, .. } = &mut insufficient {
        spec.build_cost = vec![MaterialStack::new("steel", 11)];
    }
    assert!(PreparedFactoryLifecycle::prepare(&state, &insufficient, state.time).is_err());
    assert_eq!(state, before);
    PreparedFactoryLifecycle::prepare(&state, &event, state.time)
        .unwrap()
        .install(&mut state);
    assert_eq!(state.materials["steel"], 8);
    assert_eq!(
        state.material_ledgers[&MaterialLedgerId::world()]["steel"],
        8
    );
}

#[test]
fn recipe_started_cost_preflight_reads_legacy_world_materials() {
    let mut world = World::new();
    register(&mut world);
    world.set_material_balance("steel", 2).unwrap();
    let started = DomainEvent::FactoryBuildStarted {
        job_id: 1,
        builder_agent_id: "actor".into(),
        site_id: "site".into(),
        spec: spec(),
        consume_ledger: MaterialLedgerId::world(),
        ready_at: world.state().time + 1,
        contract_version: Some(0),
        site_authority_revision: None,
        site_location_id: None,
        location_anchor_revision: None,
        construction_power_obligation: None,
    };
    world
        .append_event_for_test(WorldEventBody::Domain(started), None)
        .unwrap();
    let mut state = world.state().clone();
    state.time += 1;
    let built = DomainEvent::FactoryBuilt {
        job_id: 1,
        builder_agent_id: "actor".into(),
        site_id: "site".into(),
        spec: spec(),
    };
    PreparedFactoryLifecycle::prepare(&state, &built, state.time)
        .unwrap()
        .install(&mut state);
    state.materials.insert("ore".into(), 4);
    state.material_ledgers.remove(&MaterialLedgerId::world());
    state
        .agents
        .get_mut("actor")
        .unwrap()
        .state
        .resources
        .set(ResourceKind::Electricity, 10)
        .unwrap();
    let event = DomainEvent::RecipeStarted {
        job_id: 7002,
        requester_agent_id: "actor".into(),
        factory_id: "factory".into(),
        recipe_id: "legacy-smelt".into(),
        accepted_batches: 1,
        consume: vec![MaterialStack::new("ore", 1)],
        produce: vec![MaterialStack::new("ingot", 1)],
        byproducts: vec![],
        power_required: 1,
        power_owner_agent_id: Some("actor".into()),
        duration_ticks: 1,
        consume_ledger: MaterialLedgerId::world(),
        output_ledger: MaterialLedgerId::world(),
        bottleneck_tags: vec![],
        market_quotes: vec![],
        logistics_route_ids: vec![],
        logistics_path_ids: vec![],
        ready_at: state.time + 1,
    };
    let before = state.clone();
    let mut insufficient = event.clone();
    if let DomainEvent::RecipeStarted { consume, .. } = &mut insufficient {
        *consume = vec![MaterialStack::new("ore", 5)];
    }
    assert!(PreparedRecipeLifecycle::prepare(&state, &insufficient, state.time).is_err());
    assert_eq!(state, before);
    PreparedRecipeLifecycle::prepare(&state, &event, state.time)
        .unwrap()
        .install(&mut state);
    assert_eq!(state.materials["ore"], 3);
    assert_eq!(state.material_ledgers[&MaterialLedgerId::world()]["ore"], 3);
}
