use super::*;
use agent_world::simulator::MaterialKind;
use agent_world::simulator::{
    chunk_bounds, chunk_coords, ChunkCoord, ChunkState, FragmentResourceBudget, ModuleVisualAnchor,
    ModuleVisualEntity, PowerEvent, ResourceOwner, SpaceConfig, WorldEventKind, CHUNK_SIZE_X_CM,
    CHUNK_SIZE_Y_CM,
};

const FACILITY_MARKER_LATERAL_OFFSET: f32 = 0.9;
const FACILITY_MARKER_VERTICAL_OFFSET: f32 = 0.45;
const ASSET_MARKER_VERTICAL_OFFSET: f32 = 1.1;
const ASSET_MARKER_RING_RADIUS: f32 = 0.45;
const MODULE_VISUAL_VERTICAL_OFFSET: f32 = 1.4;
const MODULE_VISUAL_RING_RADIUS: f32 = 0.7;
const LOCATION_DEPLETION_MIN_RADIUS_FACTOR: f32 = 0.24;
const AGENT_HEIGHT_MIN_M: f32 = 0.25;
const AGENT_HEIGHT_MAX_M: f32 = 4.0;
const AGENT_BODY_RADIUS_RATIO: f32 = 0.22;
const AGENT_BODY_LENGTH_RATIO: f32 = 0.56;
const AGENT_MODULE_MARKER_MAX: usize = 16;
const AGENT_MODULE_MARKERS_PER_RING: usize = 8;
const AGENT_MODULE_RING_BASE_MULTIPLIER: f32 = 3.05;
const AGENT_MODULE_RING_GAP_RATIO: f32 = 1.36;
const AGENT_MODULE_MARKER_WIDTH_RATIO: f32 = 0.96;
const AGENT_MODULE_MARKER_HEIGHT_RATIO: f32 = 1.08;
const AGENT_MODULE_MARKER_DEPTH_RATIO: f32 = 0.82;
const AGENT_MODULE_MARKER_MIN_WIDTH: f32 = 0.28;
const AGENT_MODULE_MARKER_MIN_HEIGHT: f32 = 0.34;
const AGENT_MODULE_MARKER_MIN_DEPTH: f32 = 0.24;
const AGENT_MODULE_MARKER_WORLD_MIN_WIDTH: f32 = 0.36;
const AGENT_MODULE_MARKER_WORLD_MIN_HEIGHT: f32 = 0.44;
const AGENT_MODULE_MARKER_WORLD_MIN_DEPTH: f32 = 0.32;
const AGENT_MODULE_LAYOUT_PRIMARY_SLOTS: [(i32, i32, i32); 16] = [
    (0, 4, 2),
    (0, 3, 2),
    (-1, 3, 2),
    (1, 3, 2),
    (-1, 2, 2),
    (1, 2, 2),
    (-2, 2, 1),
    (2, 2, 1),
    (-2, 1, 1),
    (2, 1, 1),
    (-1, 1, 1),
    (1, 1, 1),
    (-1, 0, 1),
    (1, 0, 1),
    (-1, -1, 1),
    (1, -1, 1),
];

#[derive(Component)]
pub(super) struct AgentMarker {
    pub id: String,
    pub module_count: usize,
}

#[derive(Component)]
pub(super) struct LocationMarker {
    pub id: String,
    pub name: String,
    pub material: MaterialKind,
    pub radiation_emission_per_tick: i64,
}

#[derive(Component)]
pub(super) struct AssetMarker {
    pub id: String,
}

#[derive(Component)]
pub(super) struct PowerPlantMarker {
    pub id: String,
}

#[derive(Component)]
pub(super) struct PowerStorageMarker {
    pub id: String,
}

#[derive(Component)]
pub(super) struct ChunkMarker {
    pub id: String,
    pub state: String,
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
    pub pick_y: f32,
}

#[derive(Component)]
pub(super) struct TwoDMapMarker;

pub(super) fn attach_to_scene_root(commands: &mut Commands, scene: &Viewer3dScene, entity: Entity) {
    if let Some(root) = scene.root_entity {
        commands.entity(root).add_child(entity);
    }
}

pub(super) fn rebuild_scene_from_snapshot(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    snapshot: &WorldSnapshot,
) {
    for entity in scene
        .agent_entities
        .values()
        .chain(scene.location_entities.values())
        .chain(scene.asset_entities.values())
        .chain(scene.module_visual_entities.values())
        .chain(scene.power_plant_entities.values())
        .chain(scene.power_storage_entities.values())
        .chain(
            scene
                .chunk_line_entities
                .values()
                .flat_map(|items| items.iter()),
        )
        .chain(scene.background_entities.iter())
        .chain(scene.heat_overlay_entities.iter())
        .chain(scene.flow_overlay_entities.iter())
    {
        commands.entity(*entity).despawn();
    }

    scene.agent_entities.clear();
    scene.agent_positions.clear();
    scene.agent_heights_cm.clear();
    scene.agent_location_ids.clear();
    scene.agent_module_counts.clear();
    scene.location_entities.clear();
    scene.asset_entities.clear();
    scene.module_visual_entities.clear();
    scene.power_plant_entities.clear();
    scene.power_storage_entities.clear();
    scene.chunk_entities.clear();
    scene.chunk_line_entities.clear();
    scene.location_positions.clear();
    scene.location_radii_cm.clear();
    scene.background_entities.clear();
    scene.heat_overlay_entities.clear();
    scene.flow_overlay_entities.clear();
    scene.floating_origin_offset = Vec3::ZERO;

    let origin = space_origin(&snapshot.config.space);
    scene.origin = Some(origin);
    scene.space = Some(snapshot.config.space.clone());
    spawn_world_background(commands, config, assets, scene, snapshot);

    for (location_id, location) in snapshot.model.locations.iter() {
        let visual_radius_cm = location_visual_radius_cm(
            location.profile.radius_cm,
            location.fragment_budget.as_ref(),
        );
        spawn_location_entity_with_radiation(
            commands,
            config,
            assets,
            scene,
            origin,
            location_id,
            &location.name,
            location.pos,
            location.profile.material,
            visual_radius_cm,
            location.profile.radiation_emission_per_tick,
        );

        if let (Some(fragment_profile), Some(entity)) = (
            location.fragment_profile.as_ref(),
            scene.location_entities.get(location_id).copied(),
        ) {
            commands.entity(entity).with_children(|parent| {
                location_fragment_render::spawn_location_fragment_elements(
                    parent,
                    assets,
                    location_id,
                    visual_radius_cm,
                    fragment_profile,
                );
            });
        }
    }

    let module_counts = agent_module_counts_in_snapshot(snapshot);
    for (agent_id, agent) in snapshot.model.agents.iter() {
        let module_count = module_counts
            .get(agent_id.as_str())
            .copied()
            .unwrap_or_else(default_agent_module_count_estimate);
        spawn_agent_entity(
            commands,
            config,
            assets,
            scene,
            origin,
            agent_id,
            Some(agent.location_id.as_str()),
            agent.pos,
            agent.body.height_cm,
            module_count,
        );
    }

    for (facility_id, plant) in snapshot.model.power_plants.iter() {
        if let Some(location) = snapshot.model.locations.get(&plant.location_id) {
            spawn_power_plant_entity(
                commands,
                config,
                assets,
                scene,
                origin,
                facility_id,
                plant.location_id.as_str(),
                location.pos,
            );
        }
    }

    for (facility_id, storage) in snapshot.model.power_storages.iter() {
        if let Some(location) = snapshot.model.locations.get(&storage.location_id) {
            spawn_power_storage_entity(
                commands,
                config,
                assets,
                scene,
                origin,
                facility_id,
                storage.location_id.as_str(),
                location.pos,
            );
        }
    }

    for (asset_id, asset) in snapshot.model.assets.iter() {
        if let Some(anchor) = owner_anchor_pos(snapshot, &asset.owner) {
            spawn_asset_entity(commands, config, assets, scene, origin, asset_id, anchor);
        }
    }

    for module_entity in snapshot.model.module_visual_entities.values() {
        if let Some(anchor) = module_visual_anchor_pos_in_snapshot(snapshot, &module_entity.anchor)
        {
            spawn_module_visual_entity(
                commands,
                config,
                assets,
                scene,
                origin,
                module_entity,
                anchor,
            );
        }
    }

    for coord in chunk_coords(&snapshot.config.space) {
        let state = snapshot
            .model
            .chunks
            .get(&coord)
            .copied()
            .unwrap_or(ChunkState::Unexplored);
        spawn_chunk_entity(
            commands,
            config,
            assets,
            scene,
            origin,
            coord,
            state,
            &snapshot.config.space,
        );
    }
}

pub(super) fn apply_events_to_scene(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    _snapshot_time: u64,
    events: &[WorldEvent],
) {
    let Some(origin) = scene.origin else {
        return;
    };
    let Some(space) = scene.space.clone() else {
        return;
    };

    let mut last_event_id = scene.last_event_id;
    let mut processed = false;

    for event in events {
        if let Some(last_id) = last_event_id {
            if event.id <= last_id {
                continue;
            }
        }

        match &event.kind {
            WorldEventKind::LocationRegistered {
                location_id,
                name,
                pos,
                profile,
            } => {
                spawn_location_entity_with_radiation(
                    commands,
                    config,
                    assets,
                    scene,
                    origin,
                    location_id,
                    name,
                    *pos,
                    profile.material,
                    profile.radius_cm,
                    profile.radiation_emission_per_tick,
                );
            }
            WorldEventKind::AgentRegistered { agent_id, pos, .. } => {
                let height_cm = scene
                    .agent_heights_cm
                    .get(agent_id)
                    .copied()
                    .unwrap_or(agent_height_cm(None));
                let location_id = scene.agent_location_ids.get(agent_id.as_str()).cloned();
                spawn_agent_entity(
                    commands,
                    config,
                    assets,
                    scene,
                    origin,
                    agent_id,
                    location_id.as_deref(),
                    *pos,
                    height_cm,
                    scene
                        .agent_module_counts
                        .get(agent_id.as_str())
                        .copied()
                        .unwrap_or(0),
                );
            }
            WorldEventKind::AgentMoved { agent_id, to, .. } => {
                if let Some(pos) = scene.location_positions.get(to) {
                    let height_cm = scene
                        .agent_heights_cm
                        .get(agent_id)
                        .copied()
                        .unwrap_or(agent_height_cm(None));
                    scene
                        .agent_location_ids
                        .insert(agent_id.to_string(), to.to_string());
                    spawn_agent_entity(
                        commands,
                        config,
                        assets,
                        scene,
                        origin,
                        agent_id,
                        Some(to.as_str()),
                        *pos,
                        height_cm,
                        scene
                            .agent_module_counts
                            .get(agent_id.as_str())
                            .copied()
                            .unwrap_or(0),
                    );
                }
            }
            WorldEventKind::ModuleVisualEntityUpserted { entity } => {
                if let Some(anchor) = module_visual_anchor_pos_in_scene(scene, &entity.anchor) {
                    spawn_module_visual_entity(
                        commands, config, assets, scene, origin, entity, anchor,
                    );
                }
            }
            WorldEventKind::ModuleVisualEntityRemoved { entity_id } => {
                if let Some(entity) = scene.module_visual_entities.remove(entity_id.as_str()) {
                    commands.entity(entity).despawn();
                }
            }
            WorldEventKind::ChunkGenerated { coord, .. } => {
                spawn_chunk_entity(
                    commands,
                    config,
                    assets,
                    scene,
                    origin,
                    *coord,
                    ChunkState::Generated,
                    &space,
                );
            }
            WorldEventKind::Power(power_event) => match power_event {
                PowerEvent::PowerPlantRegistered { plant } => {
                    if let Some(pos) = scene.location_positions.get(&plant.location_id) {
                        spawn_power_plant_entity(
                            commands,
                            config,
                            assets,
                            scene,
                            origin,
                            &plant.id,
                            &plant.location_id,
                            *pos,
                        );
                    }
                }
                PowerEvent::PowerStorageRegistered { storage } => {
                    if let Some(pos) = scene.location_positions.get(&storage.location_id) {
                        spawn_power_storage_entity(
                            commands,
                            config,
                            assets,
                            scene,
                            origin,
                            &storage.id,
                            &storage.location_id,
                            *pos,
                        );
                    }
                }
                _ => {}
            },
            _ => {}
        }

        last_event_id = Some(event.id);
        processed = true;
    }

    if processed {
        scene.last_event_id = last_event_id;
    }
}

pub(super) fn spawn_world_background(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    snapshot: &WorldSnapshot,
) {
    let space = &snapshot.config.space;
    let world_width = (space.width_cm as f32 * config.effective_cm_to_unit()).max(WORLD_MIN_AXIS);
    let world_depth = (space.depth_cm as f32 * config.effective_cm_to_unit()).max(WORLD_MIN_AXIS);
    let world_height = (space.height_cm as f32 * config.effective_cm_to_unit()).max(WORLD_MIN_AXIS);

    let floor_entity = commands
        .spawn((
            Mesh3d(assets.world_box_mesh.clone()),
            MeshMaterial3d(assets.world_floor_material.clone()),
            Transform::from_translation(Vec3::new(
                0.0,
                -world_height * 0.5 - WORLD_FLOOR_THICKNESS * 0.5,
                0.0,
            ))
            .with_scale(Vec3::new(world_width, WORLD_FLOOR_THICKNESS, world_depth)),
            WorldFloorSurface,
            Name::new("world:floor"),
            BaseScale(Vec3::new(world_width, WORLD_FLOOR_THICKNESS, world_depth)),
        ))
        .id();
    attach_to_scene_root(commands, scene, floor_entity);
    scene.background_entities.push(floor_entity);

    let bounds_entity = commands
        .spawn((
            Mesh3d(assets.world_box_mesh.clone()),
            MeshMaterial3d(assets.world_bounds_material.clone()),
            Transform::from_scale(Vec3::new(world_width, world_height, world_depth)),
            WorldBoundsSurface,
            Name::new("world:bounds"),
            BaseScale(Vec3::new(world_width, world_height, world_depth)),
        ))
        .id();
    attach_to_scene_root(commands, scene, bounds_entity);
    scene.background_entities.push(bounds_entity);

    spawn_world_grid(
        commands,
        assets,
        scene,
        space,
        config.effective_cm_to_unit(),
        world_height,
    );
}

pub(super) fn spawn_world_grid(
    commands: &mut Commands,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    space: &SpaceConfig,
    cm_to_unit: f32,
    world_height: f32,
) {
    let thickness = grid_line_thickness(GridLineKind::World, ViewerCameraMode::TwoD);
    let y = -world_height * 0.5 + thickness * 0.5;

    let mut x_idx: usize = 0;
    for x_cm in grid_positions_cm(space.width_cm, ChunkAxis::X) {
        let x = (x_cm as f32 - space.width_cm as f32 * 0.5) * cm_to_unit;
        let world_depth = (space.depth_cm as f32 * cm_to_unit).max(WORLD_MIN_AXIS);
        let x_line = commands
            .spawn((
                Mesh3d(assets.world_box_mesh.clone()),
                MeshMaterial3d(assets.world_grid_material.clone()),
                Transform::from_translation(Vec3::new(x, y, 0.0)).with_scale(grid_line_scale(
                    GridLineAxis::AlongZ,
                    world_depth,
                    thickness,
                )),
                Name::new(format!("world:grid:x:{x_idx}")),
                BaseScale(grid_line_scale(
                    GridLineAxis::AlongZ,
                    world_depth,
                    thickness,
                )),
                GridLineVisual {
                    kind: GridLineKind::World,
                    axis: GridLineAxis::AlongZ,
                    span: world_depth,
                },
            ))
            .id();
        attach_to_scene_root(commands, scene, x_line);
        scene.background_entities.push(x_line);
        x_idx += 1;
    }

    let mut z_idx: usize = 0;
    for z_cm in grid_positions_cm(space.depth_cm, ChunkAxis::Z) {
        let z = (z_cm as f32 - space.depth_cm as f32 * 0.5) * cm_to_unit;
        let world_width = (space.width_cm as f32 * cm_to_unit).max(WORLD_MIN_AXIS);
        let z_line = commands
            .spawn((
                Mesh3d(assets.world_box_mesh.clone()),
                MeshMaterial3d(assets.world_grid_material.clone()),
                Transform::from_translation(Vec3::new(0.0, y, z)).with_scale(grid_line_scale(
                    GridLineAxis::AlongX,
                    world_width,
                    thickness,
                )),
                Name::new(format!("world:grid:z:{z_idx}")),
                BaseScale(grid_line_scale(
                    GridLineAxis::AlongX,
                    world_width,
                    thickness,
                )),
                GridLineVisual {
                    kind: GridLineKind::World,
                    axis: GridLineAxis::AlongX,
                    span: world_width,
                },
            ))
            .id();
        attach_to_scene_root(commands, scene, z_line);
        scene.background_entities.push(z_line);
        z_idx += 1;
    }
}

fn grid_positions_cm(axis_cm: i64, axis: ChunkAxis) -> Vec<i64> {
    if axis_cm <= 0 {
        return vec![0];
    }
    let step_cm = grid_step_cm_for_axis(axis);
    let mut values = vec![0];
    let mut cursor = 0_i64;
    while cursor < axis_cm {
        cursor = (cursor + step_cm).min(axis_cm);
        if values.last().copied().unwrap_or(-1) != cursor {
            values.push(cursor);
        }
    }
    values
}

fn grid_step_cm_for_axis(axis: ChunkAxis) -> i64 {
    match axis {
        ChunkAxis::X => CHUNK_SIZE_X_CM,
        ChunkAxis::Z => CHUNK_SIZE_Y_CM,
    }
}

#[derive(Clone, Copy)]
enum ChunkAxis {
    X,
    Z,
}

pub(super) fn spawn_location_entity_with_radiation(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    _assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    location_id: &str,
    name: &str,
    pos: GeoPos,
    material: MaterialKind,
    radius_cm: i64,
    radiation_emission_per_tick: i64,
) {
    scene
        .location_positions
        .insert(location_id.to_string(), pos);
    scene
        .location_radii_cm
        .insert(location_id.to_string(), radius_cm.max(1));

    let radius_world_units = location_render_radius_units(radius_cm, config.effective_cm_to_unit());
    let marker_scale = Vec3::splat(radius_world_units);
    let translation = geo_to_vec3(pos, origin, config.effective_cm_to_unit());
    if let Some(entity) = scene.location_entities.get(location_id) {
        commands.entity(*entity).insert((
            Transform::from_translation(translation).with_scale(marker_scale),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
                material,
                radiation_emission_per_tick,
            },
            BaseScale(marker_scale),
        ));
        commands
            .entity(*entity)
            .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
        commands.entity(*entity).despawn_children();
        return;
    }

    let entity = commands
        .spawn((
            Transform::from_translation(translation).with_scale(marker_scale),
            Name::new(format!("location:anchor:{location_id}:{name}")),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
                material,
                radiation_emission_per_tick,
            },
            BaseScale(marker_scale),
        ))
        .id();
    attach_to_scene_root(commands, scene, entity);
    scene
        .location_entities
        .insert(location_id.to_string(), entity);
}

pub(super) fn spawn_agent_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    agent_id: &str,
    location_id: Option<&str>,
    pos: GeoPos,
    height_cm: i64,
    module_count: usize,
) {
    scene.agent_positions.insert(agent_id.to_string(), pos);
    scene
        .agent_heights_cm
        .insert(agent_id.to_string(), height_cm.max(1));
    scene
        .agent_module_counts
        .insert(agent_id.to_string(), module_count);
    if let Some(location_id) = location_id {
        scene
            .agent_location_ids
            .insert(agent_id.to_string(), location_id.to_string());
    }

    if !config.show_agents {
        return;
    }

    let cm_to_unit = config.effective_cm_to_unit();
    let body_scale = agent_body_scale(height_cm, cm_to_unit);
    let marker_scale = agent_module_marker_scale(height_cm, cm_to_unit);
    let marker_world_scale = agent_module_marker_world_scale(marker_scale, cm_to_unit, height_cm);
    let module_markers = agent_module_marker_transforms(height_cm, module_count, cm_to_unit);
    let translation = agent_translation_for_render(scene, config, origin, agent_id, pos, height_cm);
    if let Some(entity) = scene.agent_entities.get(agent_id) {
        commands.entity(*entity).insert((
            Transform::from_translation(translation),
            Visibility::Visible,
            AgentMarker {
                id: agent_id.to_string(),
                module_count,
            },
            BaseScale(Vec3::ONE),
        ));
        commands.entity(*entity).despawn_children();
        commands.entity(*entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(assets.agent_mesh.clone()),
                MeshMaterial3d(assets.agent_material.clone()),
                Transform::from_scale(body_scale),
                Name::new(format!("agent:body:{agent_id}")),
            ));
            spawn_label(
                parent,
                assets,
                agent_id.to_string(),
                agent_label_offset(height_cm, cm_to_unit),
                format!("label:agent:{agent_id}"),
            );
            spawn_agent_two_d_map_marker(
                parent,
                assets,
                agent_id,
                height_cm,
                module_count,
                cm_to_unit,
            );
            for (marker_idx, marker_translation) in module_markers.iter().enumerate() {
                parent.spawn((
                    Mesh3d(assets.agent_module_marker_mesh.clone()),
                    MeshMaterial3d(assets.agent_module_marker_material.clone()),
                    Transform::from_translation(*marker_translation).with_scale(marker_world_scale),
                    Name::new(format!("agent:module_marker:{agent_id}:{marker_idx}")),
                ));
            }
        });
        return;
    }

    let entity = commands
        .spawn((
            Transform::from_translation(translation),
            Visibility::Visible,
            Name::new(format!("agent:{agent_id}")),
            AgentMarker {
                id: agent_id.to_string(),
                module_count,
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    attach_to_scene_root(commands, scene, entity);
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(assets.agent_mesh.clone()),
            MeshMaterial3d(assets.agent_material.clone()),
            Transform::from_scale(body_scale),
            Name::new(format!("agent:body:{agent_id}")),
        ));
        spawn_label(
            parent,
            assets,
            agent_id.to_string(),
            agent_label_offset(height_cm, cm_to_unit),
            format!("label:agent:{agent_id}"),
        );
        spawn_agent_two_d_map_marker(
            parent,
            assets,
            agent_id,
            height_cm,
            module_count,
            cm_to_unit,
        );
        for (marker_idx, marker_translation) in module_markers.iter().enumerate() {
            parent.spawn((
                Mesh3d(assets.agent_module_marker_mesh.clone()),
                MeshMaterial3d(assets.agent_module_marker_material.clone()),
                Transform::from_translation(*marker_translation).with_scale(marker_world_scale),
                Name::new(format!("agent:module_marker:{agent_id}:{marker_idx}")),
            ));
        }
    });
    scene.agent_entities.insert(agent_id.to_string(), entity);
}

fn owner_anchor_pos(snapshot: &WorldSnapshot, owner: &ResourceOwner) -> Option<GeoPos> {
    match owner {
        ResourceOwner::Agent { agent_id } => {
            snapshot.model.agents.get(agent_id).map(|agent| agent.pos)
        }
        ResourceOwner::Location { location_id } => snapshot
            .model
            .locations
            .get(location_id)
            .map(|location| location.pos),
    }
}

fn location_render_radius_units(radius_cm: i64, cm_to_unit: f32) -> f32 {
    (radius_cm.max(1) as f32) * cm_to_unit.max(f32::EPSILON)
}

fn world_units_per_meter(cm_to_unit: f32) -> f32 {
    cm_to_unit.max(f32::EPSILON) * 100.0
}

pub(super) fn location_visual_radius_cm(
    radius_cm: i64,
    fragment_budget: Option<&FragmentResourceBudget>,
) -> i64 {
    let base_radius = radius_cm.max(1);
    let Some(fragment_budget) = fragment_budget else {
        return base_radius;
    };
    let Some(remaining_ratio) = location_remaining_mass_ratio(fragment_budget) else {
        return base_radius;
    };

    let radius_factor = remaining_ratio
        .clamp(0.0, 1.0)
        .cbrt()
        .max(LOCATION_DEPLETION_MIN_RADIUS_FACTOR);
    ((base_radius as f32) * radius_factor).round().max(1.0) as i64
}

fn location_remaining_mass_ratio(fragment_budget: &FragmentResourceBudget) -> Option<f32> {
    let total_mass = fragment_budget
        .total_by_element_g
        .values()
        .copied()
        .filter(|amount| *amount > 0)
        .fold(0_i64, |acc, amount| acc.saturating_add(amount));
    if total_mass <= 0 {
        return None;
    }

    let remaining_mass = fragment_budget
        .remaining_by_element_g
        .values()
        .copied()
        .filter(|amount| *amount > 0)
        .fold(0_i64, |acc, amount| acc.saturating_add(amount));
    let clamped_remaining = remaining_mass.clamp(0, total_mass);
    Some((clamped_remaining as f32 / total_mass as f32).clamp(0.0, 1.0))
}

fn agent_height_cm(height_cm: Option<i64>) -> i64 {
    height_cm.unwrap_or(agent_world::models::DEFAULT_AGENT_HEIGHT_CM)
}

fn agent_body_scale(height_cm: i64, cm_to_unit: f32) -> Vec3 {
    let height_m = (agent_height_cm(Some(height_cm)) as f32 / 100.0)
        .clamp(AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M);
    let radius_m = (height_m * AGENT_BODY_RADIUS_RATIO).clamp(0.06, 0.9);
    let body_length_m = (height_m * AGENT_BODY_LENGTH_RATIO).max(radius_m * 0.1);
    let units_per_m = world_units_per_meter(cm_to_unit);
    Vec3::new(
        radius_m * 2.0 * units_per_m,
        body_length_m * units_per_m,
        radius_m * 2.0 * units_per_m,
    )
}

fn body_half_height_units(height_cm: i64, cm_to_unit: f32) -> f32 {
    let scale = agent_body_scale(height_cm, cm_to_unit);
    scale.y * 0.5 + scale.x * 0.5
}

fn agent_translation_for_render(
    scene: &Viewer3dScene,
    config: &Viewer3dConfig,
    origin: GeoPos,
    agent_id: &str,
    pos: GeoPos,
    height_cm: i64,
) -> Vec3 {
    let cm_to_unit = config.effective_cm_to_unit();
    let base = geo_to_vec3(pos, origin, cm_to_unit);
    let body_half_height = body_half_height_units(height_cm, cm_to_unit);
    let Some(location_id) = scene.agent_location_ids.get(agent_id) else {
        return base + Vec3::Y * body_half_height;
    };
    let Some(location_radius_cm) = scene.location_radii_cm.get(location_id.as_str()).copied()
    else {
        return base + Vec3::Y * body_half_height;
    };

    let Some(location_pos) = scene.location_positions.get(location_id.as_str()).copied() else {
        return base + Vec3::Y * body_half_height;
    };

    let location_center = geo_to_vec3(location_pos, origin, cm_to_unit);
    let location_radius = location_render_radius_units(location_radius_cm, cm_to_unit);
    let radial_offset = base - location_center;
    let surface_normal = if radial_offset.length_squared() > 1e-6 {
        radial_offset.normalize()
    } else {
        let angle = id_hash_fraction(agent_id) * std::f32::consts::TAU;
        Vec3::new(angle.cos(), 0.24, angle.sin()).normalize()
    };
    let surface_gap = (body_half_height * 0.01).max(0.006);
    location_center + surface_normal * (location_radius + body_half_height + surface_gap)
}

fn agent_body_radius_m(height_cm: i64) -> f32 {
    let height_m = (agent_height_cm(Some(height_cm)) as f32 / 100.0)
        .clamp(AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M);
    (height_m * AGENT_BODY_RADIUS_RATIO).clamp(0.06, 0.9)
}

fn capped_module_marker_count(module_count: usize) -> usize {
    module_count.min(AGENT_MODULE_MARKER_MAX)
}

fn agent_module_marker_scale(height_cm: i64, cm_to_unit: f32) -> Vec3 {
    let radius = agent_body_radius_m(height_cm);
    let units_per_m = world_units_per_meter(cm_to_unit);
    Vec3::new(
        (radius * AGENT_MODULE_MARKER_WIDTH_RATIO).clamp(AGENT_MODULE_MARKER_MIN_WIDTH, 0.62)
            * units_per_m,
        (radius * AGENT_MODULE_MARKER_HEIGHT_RATIO).clamp(AGENT_MODULE_MARKER_MIN_HEIGHT, 0.78)
            * units_per_m,
        (radius * AGENT_MODULE_MARKER_DEPTH_RATIO).clamp(AGENT_MODULE_MARKER_MIN_DEPTH, 0.42)
            * units_per_m,
    )
}

fn agent_module_marker_world_scale(
    marker_scale: Vec3,
    cm_to_unit: f32,
    base_height_cm: i64,
) -> Vec3 {
    let base_height_m = (base_height_cm.max(1) as f32 / 100.0).max(f32::EPSILON);
    let marker_height_cm = marker_scale.y / cm_to_unit.max(f32::EPSILON);
    let min_height_cm = ((AGENT_MODULE_MARKER_WORLD_MIN_HEIGHT / base_height_m)
        * base_height_cm as f32)
        .max(base_height_cm as f32 * 0.16);
    let factor = if marker_height_cm >= min_height_cm {
        1.0
    } else {
        (min_height_cm / marker_height_cm).clamp(1.0, 3.4)
    };
    let units_per_m = world_units_per_meter(cm_to_unit);
    let min_world_width = AGENT_MODULE_MARKER_WORLD_MIN_WIDTH * units_per_m;
    let min_world_height = AGENT_MODULE_MARKER_WORLD_MIN_HEIGHT * units_per_m;
    let min_world_depth = AGENT_MODULE_MARKER_WORLD_MIN_DEPTH * units_per_m;

    Vec3::new(
        (marker_scale.x * factor).max(min_world_width),
        (marker_scale.y * factor).max(min_world_height),
        (marker_scale.z * factor).max(min_world_depth),
    )
}

fn agent_module_ring_radius(height_cm: i64, ring_idx: usize, cm_to_unit: f32) -> f32 {
    let radius = agent_body_radius_m(height_cm);
    let base = radius * AGENT_MODULE_RING_BASE_MULTIPLIER;
    let ring_gap = radius * AGENT_MODULE_RING_GAP_RATIO;
    (base + ring_gap * ring_idx as f32).clamp(0.25, 4.2) * world_units_per_meter(cm_to_unit)
}

fn agent_module_marker_transforms(
    height_cm: i64,
    module_count: usize,
    cm_to_unit: f32,
) -> Vec<Vec3> {
    let marker_count = capped_module_marker_count(module_count);
    if marker_count == 0 {
        return Vec::new();
    }

    let body_scale = agent_body_scale(height_cm, cm_to_unit);
    let marker_scale = agent_module_marker_scale(height_cm, cm_to_unit);
    let body_half_height = body_half_height_units(height_cm, cm_to_unit);
    let module_gap_x = marker_scale.x * 2.05;
    let module_gap_z = marker_scale.z * 2.35;
    let module_layer_gap_z = marker_scale.z * 0.95;
    let shell_offset_x = body_scale.x * 0.98 + marker_scale.x * 1.05;
    let base_y = body_half_height * 0.2;
    let mut transforms = Vec::with_capacity(marker_count);

    for slot in AGENT_MODULE_LAYOUT_PRIMARY_SLOTS
        .iter()
        .take(marker_count)
        .copied()
    {
        transforms.push(Vec3::new(
            shell_offset_x + slot.0 as f32 * module_gap_x,
            base_y + slot.2 as f32 * (marker_scale.y * 0.32),
            slot.1 as f32 * module_gap_z + slot.2 as f32 * module_layer_gap_z,
        ));
    }

    if transforms.len() >= marker_count {
        return transforms;
    }

    let mut extra_idx = 0usize;
    while transforms.len() < marker_count {
        let ring_idx = extra_idx / AGENT_MODULE_MARKERS_PER_RING;
        let within_ring = extra_idx % AGENT_MODULE_MARKERS_PER_RING;
        let remaining = marker_count - transforms.len();
        let markers_in_ring = remaining.min(AGENT_MODULE_MARKERS_PER_RING);
        let angle_step = std::f32::consts::TAU / markers_in_ring as f32;
        let angle = angle_step * within_ring as f32;
        let ring_radius = agent_module_ring_radius(height_cm, ring_idx, cm_to_unit);
        let vertical = base_y + marker_scale.y * (0.28 + ring_idx as f32 * 0.24);
        transforms.push(Vec3::new(
            shell_offset_x + angle.cos() * ring_radius,
            vertical,
            angle.sin() * ring_radius,
        ));
        extra_idx += 1;
    }

    transforms
}

pub(super) fn agent_module_counts_in_snapshot(
    snapshot: &WorldSnapshot,
) -> std::collections::HashMap<String, usize> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for module_entity in snapshot.model.module_visual_entities.values() {
        if let ModuleVisualAnchor::Agent { agent_id } = &module_entity.anchor {
            *counts.entry(agent_id.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn default_agent_module_count_estimate() -> usize {
    agent_world::models::AgentBodyState::default()
        .slots
        .iter()
        .filter(|slot| slot.installed_module.is_some())
        .count()
}

fn agent_label_offset(height_cm: i64, cm_to_unit: f32) -> f32 {
    let height_m = (agent_height_cm(Some(height_cm)) as f32 / 100.0)
        .clamp(AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M);
    (height_m * 0.65).max(AGENT_LABEL_OFFSET) * world_units_per_meter(cm_to_unit)
}

#[path = "scene_helpers_entities.rs"]
mod scene_helpers_entities;

use scene_helpers_entities::{
    id_hash_fraction, module_visual_anchor_pos_in_scene, module_visual_anchor_pos_in_snapshot,
    spawn_agent_two_d_map_marker, spawn_asset_entity, spawn_module_visual_entity,
    spawn_power_plant_entity, spawn_power_storage_entity,
};

pub(super) fn spawn_chunk_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    coord: ChunkCoord,
    state: ChunkState,
    space: &SpaceConfig,
) {
    scene_helpers_entities::spawn_chunk_entity(
        commands, config, assets, scene, origin, coord, state, space,
    );
}

pub(super) fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    assets: &Viewer3dAssets,
    text: String,
    offset_y: f32,
    name: String,
) {
    parent.spawn((
        Text2d::new(text),
        TextFont {
            font: assets.label_font.clone(),
            font_size: LABEL_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, offset_y, 0.0))
            .with_scale(Vec3::splat(LABEL_SCALE)),
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Name::new(name),
    ));
}

pub(super) fn space_origin(space: &SpaceConfig) -> GeoPos {
    GeoPos {
        x_cm: space.width_cm as f64 / 2.0,
        y_cm: space.depth_cm as f64 / 2.0,
        z_cm: space.height_cm as f64 / 2.0,
    }
}

pub(super) fn geo_to_vec3(pos: GeoPos, origin: GeoPos, cm_to_unit: f32) -> Vec3 {
    let scale = cm_to_unit as f64;
    Vec3::new(
        ((pos.x_cm - origin.x_cm) * scale) as f32,
        ((pos.z_cm - origin.z_cm) * scale) as f32,
        ((pos.y_cm - origin.y_cm) * scale) as f32,
    )
}

pub(super) fn ray_point_distance(ray: Ray3d, point: Vec3) -> Option<f32> {
    let direction = ray.direction.as_vec3();
    let to_point = point - ray.origin;
    let t = direction.dot(to_point);
    if t < 0.0 {
        return None;
    }
    let closest = ray.origin + direction * t;
    Some(closest.distance(point))
}

pub(super) fn apply_entity_highlight(
    transforms: &mut Query<(&mut Transform, Option<&BaseScale>)>,
    entity: Entity,
) {
    if let Ok((mut transform, base)) = transforms.get_mut(entity) {
        let base_scale = base.map(|scale| scale.0).unwrap_or(Vec3::ONE);
        transform.scale = base_scale * 1.6;
    }
}

pub(super) fn should_apply_scale_highlight(kind: SelectionKind) -> bool {
    !matches!(kind, SelectionKind::Fragment)
}

pub(super) fn reset_entity_scale(
    transforms: &mut Query<(&mut Transform, Option<&BaseScale>)>,
    entity: Entity,
) {
    if let Ok((mut transform, base)) = transforms.get_mut(entity) {
        let base_scale = base.map(|scale| scale.0).unwrap_or(Vec3::ONE);
        transform.scale = base_scale;
    }
}

#[cfg(test)]
mod depletion_tests {
    use super::*;
    use agent_world::simulator::FragmentElementKind;

    #[test]
    fn location_visual_radius_cm_keeps_base_when_budget_missing() {
        assert_eq!(location_visual_radius_cm(900, None), 900);
    }

    #[test]
    fn location_visual_radius_cm_tracks_remaining_mass_ratio() {
        let mut budget = FragmentResourceBudget::default();
        budget
            .total_by_element_g
            .insert(FragmentElementKind::Iron, 1_000);
        budget
            .remaining_by_element_g
            .insert(FragmentElementKind::Iron, 125);

        assert_eq!(location_visual_radius_cm(800, Some(&budget)), 400);

        budget
            .remaining_by_element_g
            .insert(FragmentElementKind::Iron, 0);
        let min_radius = location_visual_radius_cm(800, Some(&budget));
        assert_eq!(min_radius, 192);
    }

    #[test]
    fn location_render_radius_units_scales_by_world_units_without_clamp() {
        let mapped = location_render_radius_units(500_000, 0.00001);
        assert!((mapped - 5.0).abs() < f32::EPSILON);

        let tiny = location_render_radius_units(100, 0.00001);
        assert!((tiny - 0.001).abs() < f32::EPSILON);

        let large = location_render_radius_units(10_000_000, 0.00001);
        assert!((large - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn should_apply_scale_highlight_skips_fragment() {
        assert!(should_apply_scale_highlight(SelectionKind::Agent));
        assert!(should_apply_scale_highlight(SelectionKind::Location));
        assert!(!should_apply_scale_highlight(SelectionKind::Fragment));
    }
}
