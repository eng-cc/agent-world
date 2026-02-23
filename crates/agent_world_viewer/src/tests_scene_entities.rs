use super::*;

fn test_assets() -> Viewer3dAssets {
    Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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

#[test]
fn spawn_location_entity_keeps_anchor_without_label() {
    let mut app = App::new();
    app.add_systems(Update, spawn_label_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut label_query = world.query::<&Text2d>();
    assert!(label_query.iter(world).next().is_none());

    let mut marker_query = world.query::<(&LocationMarker, &Name)>();
    let (_, name) = marker_query
        .iter(world)
        .find(|(marker, _)| marker.id == "loc-1")
        .expect("location marker exists");
    assert!(name.as_str().starts_with("location:anchor:"));
}

#[test]
fn spawn_location_entity_uses_linear_anchor_radius_scale() {
    let mut app = App::new();
    app.add_systems(Update, spawn_location_scale_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let cm_to_unit = world.resource::<Viewer3dConfig>().effective_cm_to_unit();
    let mut query = world.query::<(&LocationMarker, &BaseScale)>();
    let (marker, base) = query
        .iter(world)
        .find(|(marker, _)| marker.id == "loc-scale")
        .expect("location marker exists");
    let expected = 20_000.0_f32 * cm_to_unit;
    assert!((base.0.x - expected).abs() < 1e-3);
    assert_eq!(marker.material, MaterialKind::Silicate);
    assert_eq!(marker.radiation_emission_per_tick, 0);
}

#[test]
fn spawn_location_entity_renders_fine_grained_ring_details() {
    let mut app = App::new();
    app.add_systems(Update, spawn_location_detail_ring_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    let ring_count = query
        .iter(world)
        .filter(|name| {
            name.as_str()
                .starts_with("location:detail:ring:loc-detail-ring:")
        })
        .count();
    let halo_count = query
        .iter(world)
        .filter(|name| {
            name.as_str()
                .starts_with("location:detail:halo:loc-detail-ring:")
        })
        .count();
    assert!(ring_count >= 2);
    assert_eq!(halo_count, 0);
}

#[test]
fn spawn_location_entity_renders_radiation_halo_details() {
    let mut app = App::new();
    app.add_systems(Update, spawn_location_detail_halo_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    let ring_count = query
        .iter(world)
        .filter(|name| {
            name.as_str()
                .starts_with("location:detail:ring:loc-detail-halo:")
        })
        .count();
    let halo_count = query
        .iter(world)
        .filter(|name| {
            name.as_str()
                .starts_with("location:detail:halo:loc-detail-halo:")
        })
        .count();

    assert!(ring_count >= 2);
    assert!(halo_count >= 1);
}

#[test]
fn spawn_location_entity_skips_shell_details_when_disabled() {
    let mut app = App::new();
    app.add_systems(Update, spawn_location_detail_halo_test_system);
    let mut config = Viewer3dConfig::default();
    config.assets.location_shell_enabled = false;
    app.insert_resource(config);
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    let detail_count = query
        .iter(world)
        .filter(|name| {
            name.as_str().starts_with("location:detail:")
                && name.as_str().contains("loc-detail-halo")
        })
        .count();
    assert_eq!(detail_count, 0);
}

#[test]
fn spawn_agent_entity_uses_body_height_scale() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_scale_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let units_per_m = world.resource::<Viewer3dConfig>().effective_cm_to_unit() * 100.0;
    let mut query = world.query::<(&AgentMarker, &BaseScale)>();
    let (marker, base) = query
        .iter(world)
        .find(|(marker, _)| marker.id == "agent-scale")
        .expect("agent marker exists");
    assert_eq!(marker.module_count, 5);
    assert!((base.0.y - 1.0).abs() < 1e-3);
    assert!((base.0.x - 1.0).abs() < 1e-3);

    let mut body_query = world.query::<(&Name, &Transform)>();
    let (_, body_transform) = body_query
        .iter(world)
        .find(|(name, _)| name.as_str() == "agent:body:agent-scale")
        .expect("agent body exists");
    assert!((body_transform.scale.y - (1.12 * units_per_m)).abs() < 1e-6);
    assert!((body_transform.scale.x - (0.88 * units_per_m)).abs() < 1e-6);
}

#[test]
fn spawn_agent_entity_renders_direction_and_speed_feedback_when_moving() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_motion_feedback_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(test_assets());

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&Name, &Transform)>();
    let direction_transform = query
        .iter(world)
        .find(|(name, _)| name.as_str() == "agent:direction_indicator:agent-motion")
        .map(|(_, transform)| *transform)
        .expect("direction indicator exists");
    assert_ne!(direction_transform.rotation, Quat::IDENTITY);

    let mut speed_query = world.query::<&Name>();
    assert!(speed_query
        .iter(world)
        .any(|name| name.as_str() == "agent:speed_effect:agent-motion"));
}

#[test]
fn spawn_agent_entity_skips_direction_and_speed_feedback_when_stationary() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_scale_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(test_assets());

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    assert!(!query
        .iter(world)
        .any(|name| name.as_str() == "agent:direction_indicator:agent-scale"));
    assert!(!query
        .iter(world)
        .any(|name| name.as_str() == "agent:speed_effect:agent-scale"));
}

#[test]
fn spawn_agent_entity_attaches_to_location_surface() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_surface_attachment_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut location_query = world.query::<(&LocationMarker, &Transform)>();
    let (location_translation, location_radius) = location_query
        .iter(world)
        .find(|(marker, _)| marker.id == "loc-surface")
        .map(|(_, transform)| (transform.translation, transform.scale.x))
        .expect("location marker exists");

    let mut agent_query = world.query::<(&AgentMarker, &Transform)>();
    let agent_translation = agent_query
        .iter(world)
        .find(|(marker, _)| marker.id == "agent-surface")
        .map(|(_, transform)| transform.translation)
        .expect("agent marker exists");

    let mut body_query = world.query::<(&Name, &Transform)>();
    let body_scale = body_query
        .iter(world)
        .find(|(name, _)| name.as_str() == "agent:body:agent-surface")
        .map(|(_, transform)| transform.scale)
        .expect("agent body exists");

    let body_half_height = body_scale.y * 0.5 + body_scale.x * 0.5;
    let center_distance = agent_translation.distance(location_translation);
    let surface_offset = center_distance - (location_radius + body_half_height);
    assert!(surface_offset >= 0.005);
    assert!(surface_offset <= 0.03);
}

#[test]
fn spawn_agent_entity_preserves_location_standoff_distance() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_surface_standoff_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let cm_to_unit = world.resource::<Viewer3dConfig>().effective_cm_to_unit();
    let mut location_query = world.query::<(&LocationMarker, &Transform)>();
    let (location_translation, location_radius) = location_query
        .iter(world)
        .find(|(marker, _)| marker.id == "loc-surface-standoff")
        .map(|(_, transform)| (transform.translation, transform.scale.x))
        .expect("location marker exists");

    let mut agent_query = world.query::<(&AgentMarker, &Transform)>();
    let agent_translation = agent_query
        .iter(world)
        .find(|(marker, _)| marker.id == "agent-surface-standoff")
        .map(|(_, transform)| transform.translation)
        .expect("agent marker exists");

    let mut body_query = world.query::<(&Name, &Transform)>();
    let body_scale = body_query
        .iter(world)
        .find(|(name, _)| name.as_str() == "agent:body:agent-surface-standoff")
        .map(|(_, transform)| transform.scale)
        .expect("agent body exists");

    let body_half_height = body_scale.y * 0.5 + body_scale.x * 0.5;
    let center_distance = agent_translation.distance(location_translation);
    let surface_offset = center_distance - (location_radius + body_half_height);
    let expected_standoff = 5_000.0_f32 * cm_to_unit;

    assert!(surface_offset >= expected_standoff);
    assert!(surface_offset <= expected_standoff + 0.03);
}

#[test]
fn spawn_agent_entity_renders_module_markers_up_to_cap() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_module_marker_count_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    let marker_count = query
        .iter(world)
        .filter(|name| {
            name.as_str()
                .starts_with("agent:module_marker:agent-module-cap:")
        })
        .count();
    assert_eq!(marker_count, 16);
}

#[test]
fn spawn_agent_entity_robot_layout_places_head_slot_first() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_robot_layout_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let units_per_m = world.resource::<Viewer3dConfig>().effective_cm_to_unit() * 100.0;
    let mut query = world.query::<(&Name, &Transform)>();
    let marker_transform = query
        .iter(world)
        .find(|(name, _)| name.as_str() == "agent:module_marker:agent-robot-layout:0")
        .map(|(_, transform)| *transform)
        .expect("first module marker exists");

    assert!(marker_transform.translation.x > 0.55 * units_per_m);
    assert!(marker_transform.translation.z > 0.65 * units_per_m);
}

#[test]
fn rebuild_scene_maps_agent_module_count_from_module_visual_entities() {
    let mut app = App::new();
    app.add_systems(Update, rebuild_scene_module_count_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&AgentMarker>();
    let marker = query
        .iter(world)
        .find(|marker| marker.id == "agent-modules")
        .expect("agent marker exists");
    assert_eq!(marker.module_count, 2);
}

#[test]
fn rebuild_scene_uses_default_module_count_when_no_module_visual_entities() {
    let mut app = App::new();
    app.add_systems(Update, rebuild_scene_default_module_count_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        agent_module_marker_mesh: Handle::default(),
        agent_module_marker_material: Handle::default(),
        location_mesh: Handle::default(),
        fragment_element_material_library: FragmentElementMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        location_core_silicate_material: Handle::default(),
        location_core_metal_material: Handle::default(),
        location_core_ice_material: Handle::default(),
        location_halo_material: Handle::default(),
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
    });

    app.update();

    let expected_default_count = agent_world::models::AgentBodyState::default()
        .slots
        .iter()
        .filter(|slot| slot.installed_module.is_some())
        .count();

    let world = app.world_mut();
    let mut query = world.query::<&AgentMarker>();
    let marker = query
        .iter(world)
        .find(|marker| marker.id == "agent-default-modules")
        .expect("agent marker exists");
    assert_eq!(marker.module_count, expected_default_count);
}
