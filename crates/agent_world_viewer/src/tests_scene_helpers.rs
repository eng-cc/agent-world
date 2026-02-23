use super::*;

pub(super) fn spawn_label_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-1",
        "Alpha",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        100,
        0,
        None,
    );
}

pub(super) fn spawn_location_scale_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-scale",
        "Scale",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        20_000,
        0,
        None,
    );
}

pub(super) fn spawn_location_detail_ring_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-detail-ring",
        "DetailRing",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        20_000,
        0,
        None,
    );
}

pub(super) fn spawn_location_detail_halo_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-detail-halo",
        "DetailHalo",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Metal,
        6_000,
        10_000,
        None,
    );
}

pub(super) fn spawn_location_damage_detail_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    let mut budget = agent_world::simulator::FragmentResourceBudget::default();
    budget
        .total_by_element_g
        .insert(agent_world::simulator::FragmentElementKind::Iron, 1_000);
    budget
        .remaining_by_element_g
        .insert(agent_world::simulator::FragmentElementKind::Iron, 100);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-damage",
        "Damage",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        8_000,
        1_000,
        Some(&budget),
    );
}

pub(super) fn spawn_agent_scale_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-scale",
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        200,
        5,
        None,
    );
}

pub(super) fn spawn_agent_surface_attachment_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-surface",
        "Surface",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        240,
        0,
        None,
    );
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-surface",
        Some("loc-surface"),
        GeoPos::new(0.0, 0.0, 0.0),
        100,
        6,
        None,
    );
}

pub(super) fn spawn_agent_surface_standoff_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-surface-standoff",
        "SurfaceStandoff",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        240,
        0,
        None,
    );
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-surface-standoff",
        Some("loc-surface-standoff"),
        GeoPos::new(0.0, 0.0, 5_240.0),
        100,
        6,
        None,
    );
}

pub(super) fn spawn_agent_module_marker_count_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-module-cap",
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        180,
        24,
        None,
    );
}

pub(super) fn spawn_agent_robot_layout_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-robot-layout",
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        180,
        8,
        None,
    );
}

pub(super) fn spawn_agent_motion_feedback_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    let kinematics = agent_world::simulator::AgentKinematics {
        speed_cm_per_tick: 320_000,
        move_target_location_id: None,
        move_target: Some(GeoPos::new(10_000.0, 0.0, 0.0)),
        move_started_at_tick: Some(12),
        move_eta_tick: Some(13),
        move_remaining_cm: 10_000,
    };
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-motion",
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        170,
        4,
        Some(&kinematics),
    );
}

pub(super) fn rebuild_scene_module_count_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let mut model = agent_world::simulator::WorldModel::default();
    model.agents.insert(
        "agent-modules".to_string(),
        agent_world::simulator::Agent::new("agent-modules", "loc-1", GeoPos::new(0.0, 0.0, 0.0)),
    );
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new("loc-1", "Loc", GeoPos::new(0.0, 0.0, 0.0)),
    );
    model.module_visual_entities.insert(
        "mv-1".to_string(),
        agent_world::simulator::ModuleVisualEntity {
            entity_id: "mv-1".to_string(),
            module_id: "m.power".to_string(),
            kind: "artifact".to_string(),
            label: None,
            anchor: agent_world::simulator::ModuleVisualAnchor::Agent {
                agent_id: "agent-modules".to_string(),
            },
        },
    );
    model.module_visual_entities.insert(
        "mv-2".to_string(),
        agent_world::simulator::ModuleVisualEntity {
            entity_id: "mv-2".to_string(),
            module_id: "m.sensor".to_string(),
            kind: "artifact".to_string(),
            label: None,
            anchor: agent_world::simulator::ModuleVisualAnchor::Agent {
                agent_id: "agent-modules".to_string(),
            },
        },
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 1,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, &snapshot);
}

pub(super) fn rebuild_scene_default_module_count_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let mut model = agent_world::simulator::WorldModel::default();
    model.agents.insert(
        "agent-default-modules".to_string(),
        agent_world::simulator::Agent::new(
            "agent-default-modules",
            "loc-1",
            GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new("loc-1", "Loc", GeoPos::new(0.0, 0.0, 0.0)),
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 1,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, &snapshot);
}
