use super::*;
use crate::viewer_3d_config::Viewer3dConfig;

#[test]
fn spawn_world_background_adds_bounds_and_chunk_sized_grid() {
    let mut app = App::new();
    app.add_systems(Update, spawn_background_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(sample_assets());

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    let names: Vec<String> = query.iter(world).map(|name| name.to_string()).collect();
    assert!(names.iter().any(|name| name == "world:bounds"));
    assert!(names.iter().any(|name| name == "world:floor"));
    assert!(names.iter().any(|name| name.starts_with("world:grid:x:")));
    assert!(names.iter().any(|name| name.starts_with("world:grid:z:")));

    let world_grid_x = names
        .iter()
        .filter(|name| name.starts_with("world:grid:x:"))
        .count();
    let world_grid_z = names
        .iter()
        .filter(|name| name.starts_with("world:grid:z:"))
        .count();
    assert_eq!(world_grid_x, 6);
    assert_eq!(world_grid_z, 6);
}

#[test]
fn rebuild_scene_spawns_chunk_grid_lines_for_all_chunks() {
    let mut app = App::new();
    app.add_systems(Update, rebuild_scene_chunks_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(sample_assets());

    app.update();

    let scene = app.world().resource::<Viewer3dScene>();
    assert_eq!(scene.chunk_entities.len(), 25);
    assert_eq!(scene.chunk_line_entities.len(), 25);
    let line_count: usize = scene
        .chunk_line_entities
        .values()
        .map(|items| items.len())
        .sum();
    assert_eq!(line_count, 100);
}

fn spawn_background_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    spawn_world_background(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        &sample_snapshot(),
    );
}

fn rebuild_scene_chunks_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let snapshot = sample_snapshot();
    rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, &snapshot);
}

fn sample_snapshot() -> WorldSnapshot {
    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    let mut config = agent_world::simulator::WorldConfig::default();
    config.space.width_cm = 10_000_000;
    config.space.depth_cm = 10_000_000;
    config.space.height_cm = 1_000_000;

    WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 1,
        config,
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    }
}

fn sample_assets() -> Viewer3dAssets {
    Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        location_mesh: Handle::default(),
        location_material_library: LocationMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        chunk_unexplored_material: Handle::default(),
        chunk_generated_material: Handle::default(),
        chunk_exhausted_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        heat_low_material: Handle::default(),
        heat_mid_material: Handle::default(),
        heat_high_material: Handle::default(),
        flow_power_material: Handle::default(),
        flow_trade_material: Handle::default(),
        label_font: Handle::default(),
    }
}
