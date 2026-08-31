//! RED contract for the canonical typed `WorldStateProjection`.
//!
//! The projection must borrow the canonical state when no overlay is present,
//! preserving the exact serde representation and hash.  A typed body overlay
//! may change only the selected agent's body view and activity timestamp in
//! the projection; the borrowed canonical state must remain untouched.

use super::super::{Action, BodyOverlay, World, WorldStateProjection};
use super::pos;
use crate::models::BodyKernelView;
use crate::runtime::util::hash_json;

fn registered_agent_world() -> World {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "projection-fixture-agent".to_string(),
        pos: pos(7, -11),
    });
    world
        .step()
        .expect("register the real projection fixture agent");
    world
}

#[test]
fn borrowed_no_overlay_projection_preserves_world_state_bytes_and_hash() {
    let world = registered_agent_world();

    let direct_bytes = serde_json::to_vec(world.state()).expect("serialize direct WorldState");
    let direct_hash = hash_json(world.state()).expect("hash direct WorldState");

    let projection = WorldStateProjection::borrowed(world.state());
    let projected_bytes = serde_json::to_vec(&projection)
        .expect("serialize borrowed no-overlay WorldStateProjection");
    let projected_hash = hash_json(&projection).expect("hash borrowed no-overlay projection");

    assert_eq!(
        projected_bytes, direct_bytes,
        "a borrowed no-overlay projection must preserve the canonical WorldState serde bytes"
    );
    assert_eq!(
        projected_hash, direct_hash,
        "a borrowed no-overlay projection must preserve the canonical WorldState hash"
    );
}

#[test]
fn typed_body_overlay_matches_golden_state_without_mutating_original() {
    let world = registered_agent_world();
    let agent_id = "projection-fixture-agent";
    let body_view = BodyKernelView {
        mass_kg: 321,
        radius_cm: 144,
        thrust_limit: 987,
        cross_section_cm2: 12_345,
    };
    let last_active = world.state().time.saturating_add(37);

    let original_bytes = serde_json::to_vec(world.state()).expect("serialize original state");
    let original_body_view = world
        .state()
        .agents
        .get(agent_id)
        .expect("fixture agent exists")
        .state
        .body_view
        .clone();
    let original_last_active = world
        .state()
        .agents
        .get(agent_id)
        .expect("fixture agent exists")
        .last_active;

    // Clone is deliberately restricted to the expected-result oracle.  The
    // production projection must borrow `world.state()` and apply a typed
    // overlay without cloning or mutating canonical state.
    let mut golden = world.state().clone();
    let golden_agent = golden
        .agents
        .get_mut(agent_id)
        .expect("fixture agent exists in golden state");
    golden_agent.state.body_view = body_view.clone();
    golden_agent.last_active = last_active;

    let projection = WorldStateProjection::borrowed(world.state())
        .with_body_overlay(BodyOverlay::new(agent_id, body_view, last_active));
    let projected_bytes = serde_json::to_vec(&projection).expect("serialize body overlay");
    let golden_bytes = serde_json::to_vec(&golden).expect("serialize golden state");

    assert_eq!(
        projected_bytes, golden_bytes,
        "typed body overlay must serialize exactly like the equivalent golden state"
    );
    assert_eq!(
        hash_json(&projection).expect("hash body overlay projection"),
        hash_json(&golden).expect("hash golden state"),
        "typed body overlay must produce the equivalent canonical hash"
    );

    assert_eq!(
        serde_json::to_vec(world.state()).expect("serialize state after projection"),
        original_bytes,
        "building a projection must not mutate the borrowed canonical state"
    );
    let original_agent = world
        .state()
        .agents
        .get(agent_id)
        .expect("fixture agent remains present");
    assert_eq!(original_agent.state.body_view, original_body_view);
    assert_eq!(original_agent.last_active, original_last_active);
}
