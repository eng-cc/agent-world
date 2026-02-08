use super::*;
use agent_world::simulator::{
    chunk_bounds, ChunkCoord, ChunkState, ModuleVisualAnchor, ModuleVisualEntity, PowerEvent,
    ResourceOwner, SpaceConfig, WorldEventKind,
};

const FACILITY_MARKER_LATERAL_OFFSET: f32 = 0.9;
const FACILITY_MARKER_VERTICAL_OFFSET: f32 = 0.45;
const ASSET_MARKER_VERTICAL_OFFSET: f32 = 1.1;
const ASSET_MARKER_RING_RADIUS: f32 = 0.45;
const MODULE_VISUAL_VERTICAL_OFFSET: f32 = 1.4;
const MODULE_VISUAL_RING_RADIUS: f32 = 0.7;
const CHUNK_MARKER_MIN_SIZE: f32 = 0.45;
const CHUNK_MARKER_MAX_SIZE: f32 = 1.8;
const CHUNK_MARKER_VERTICAL_OFFSET: f32 = 0.2;
const LOCATION_RADIUS_MIN_M: f32 = 0.25;
const LOCATION_RADIUS_MAX_M: f32 = 3000.0;
const AGENT_HEIGHT_MIN_M: f32 = 0.25;
const AGENT_HEIGHT_MAX_M: f32 = 4.0;

#[derive(Component)]
pub(super) struct AgentMarker {
    pub id: String,
}

#[derive(Component)]
pub(super) struct LocationMarker {
    pub id: String,
    pub name: String,
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
        .chain(scene.chunk_entities.values())
        .chain(scene.background_entities.iter())
        .chain(scene.heat_overlay_entities.iter())
        .chain(scene.flow_overlay_entities.iter())
    {
        commands.entity(*entity).despawn();
    }

    scene.agent_entities.clear();
    scene.agent_positions.clear();
    scene.agent_heights_cm.clear();
    scene.location_entities.clear();
    scene.asset_entities.clear();
    scene.module_visual_entities.clear();
    scene.power_plant_entities.clear();
    scene.power_storage_entities.clear();
    scene.chunk_entities.clear();
    scene.location_positions.clear();
    scene.background_entities.clear();
    scene.heat_overlay_entities.clear();
    scene.flow_overlay_entities.clear();

    let origin = space_origin(&snapshot.config.space);
    scene.origin = Some(origin);
    scene.space = Some(snapshot.config.space.clone());
    spawn_world_background(commands, config, assets, scene, snapshot);

    for (location_id, location) in snapshot.model.locations.iter() {
        spawn_location_entity(
            commands,
            config,
            assets,
            scene,
            origin,
            location_id,
            &location.name,
            location.pos,
            location.profile.radius_cm,
        );
    }

    for (agent_id, agent) in snapshot.model.agents.iter() {
        spawn_agent_entity(
            commands,
            config,
            assets,
            scene,
            origin,
            agent_id,
            agent.pos,
            agent.body.height_cm,
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

    for (coord, state) in snapshot.model.chunks.iter() {
        spawn_chunk_entity(
            commands,
            config,
            assets,
            scene,
            origin,
            *coord,
            *state,
            &snapshot.config.space,
        );
    }
}

pub(super) fn apply_events_to_scene(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    snapshot_time: u64,
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
        if event.time <= snapshot_time {
            continue;
        }
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
                spawn_location_entity(
                    commands,
                    config,
                    assets,
                    scene,
                    origin,
                    location_id,
                    name,
                    *pos,
                    profile.radius_cm,
                );
            }
            WorldEventKind::AgentRegistered { agent_id, pos, .. } => {
                let height_cm = scene
                    .agent_heights_cm
                    .get(agent_id)
                    .copied()
                    .unwrap_or(agent_height_cm(None));
                spawn_agent_entity(
                    commands, config, assets, scene, origin, agent_id, *pos, height_cm,
                );
            }
            WorldEventKind::AgentMoved { agent_id, to, .. } => {
                if let Some(pos) = scene.location_positions.get(to) {
                    let height_cm = scene
                        .agent_heights_cm
                        .get(agent_id)
                        .copied()
                        .unwrap_or(agent_height_cm(None));
                    spawn_agent_entity(
                        commands, config, assets, scene, origin, agent_id, *pos, height_cm,
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
            Name::new("world:floor"),
            BaseScale(Vec3::new(world_width, WORLD_FLOOR_THICKNESS, world_depth)),
        ))
        .id();
    scene.background_entities.push(floor_entity);

    let bounds_entity = commands
        .spawn((
            Mesh3d(assets.world_box_mesh.clone()),
            MeshMaterial3d(assets.world_bounds_material.clone()),
            Transform::from_scale(Vec3::new(world_width, world_height, world_depth)),
            Name::new("world:bounds"),
            BaseScale(Vec3::new(world_width, world_height, world_depth)),
        ))
        .id();
    scene.background_entities.push(bounds_entity);

    spawn_world_grid(
        commands,
        assets,
        scene,
        world_width,
        world_depth,
        world_height,
    );
}

pub(super) fn spawn_world_grid(
    commands: &mut Commands,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    world_width: f32,
    world_depth: f32,
    world_height: f32,
) {
    if WORLD_GRID_LINES_PER_AXIS == 0 {
        return;
    }

    let half_width = world_width * 0.5;
    let half_depth = world_depth * 0.5;
    let y = -world_height * 0.5 + WORLD_GRID_LINE_THICKNESS * 0.5;
    let steps = WORLD_GRID_LINES_PER_AXIS as f32;

    for idx in 0..=WORLD_GRID_LINES_PER_AXIS {
        let t = idx as f32 / steps;
        let x = -half_width + world_width * t;
        let x_line = commands
            .spawn((
                Mesh3d(assets.world_box_mesh.clone()),
                MeshMaterial3d(assets.world_grid_material.clone()),
                Transform::from_translation(Vec3::new(x, y, 0.0)).with_scale(Vec3::new(
                    WORLD_GRID_LINE_THICKNESS,
                    WORLD_GRID_LINE_THICKNESS,
                    world_depth,
                )),
                Name::new(format!("world:grid:x:{idx}")),
                BaseScale(Vec3::new(
                    WORLD_GRID_LINE_THICKNESS,
                    WORLD_GRID_LINE_THICKNESS,
                    world_depth,
                )),
            ))
            .id();
        scene.background_entities.push(x_line);
    }

    for idx in 0..=WORLD_GRID_LINES_PER_AXIS {
        let t = idx as f32 / steps;
        let z = -half_depth + world_depth * t;
        let z_line = commands
            .spawn((
                Mesh3d(assets.world_box_mesh.clone()),
                MeshMaterial3d(assets.world_grid_material.clone()),
                Transform::from_translation(Vec3::new(0.0, y, z)).with_scale(Vec3::new(
                    world_width,
                    WORLD_GRID_LINE_THICKNESS,
                    WORLD_GRID_LINE_THICKNESS,
                )),
                Name::new(format!("world:grid:z:{idx}")),
                BaseScale(Vec3::new(
                    world_width,
                    WORLD_GRID_LINE_THICKNESS,
                    WORLD_GRID_LINE_THICKNESS,
                )),
            ))
            .id();
        scene.background_entities.push(z_line);
    }
}

pub(super) fn spawn_location_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    location_id: &str,
    name: &str,
    pos: GeoPos,
    radius_cm: i64,
) {
    scene
        .location_positions
        .insert(location_id.to_string(), pos);

    if !config.show_locations {
        return;
    }

    let radius_m = location_radius_m(radius_cm);
    let marker_scale = Vec3::splat(radius_m);
    let translation = geo_to_vec3(pos, origin, config.effective_cm_to_unit());
    if let Some(entity) = scene.location_entities.get(location_id) {
        commands.entity(*entity).insert((
            Transform::from_translation(translation),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
            },
            BaseScale(marker_scale),
        ));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.location_mesh.clone()),
            MeshMaterial3d(assets.location_material.clone()),
            Transform::from_translation(translation).with_scale(marker_scale),
            Name::new(format!("location:{location_id}:{name}")),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
            },
            BaseScale(marker_scale),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            name.to_string(),
            location_label_offset(radius_m),
            format!("label:location:{location_id}"),
        );
    });
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
    pos: GeoPos,
    height_cm: i64,
) {
    scene.agent_positions.insert(agent_id.to_string(), pos);
    scene
        .agent_heights_cm
        .insert(agent_id.to_string(), height_cm.max(1));

    if !config.show_agents {
        return;
    }

    let marker_scale = Vec3::splat(agent_radius_m(height_cm));
    let translation = geo_to_vec3(pos, origin, config.effective_cm_to_unit());
    if let Some(entity) = scene.agent_entities.get(agent_id) {
        commands.entity(*entity).insert((
            Transform::from_translation(translation).with_scale(marker_scale),
            BaseScale(marker_scale),
        ));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.agent_mesh.clone()),
            MeshMaterial3d(assets.agent_material.clone()),
            Transform::from_translation(translation).with_scale(marker_scale),
            Name::new(format!("agent:{agent_id}")),
            AgentMarker {
                id: agent_id.to_string(),
            },
            BaseScale(marker_scale),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            agent_id.to_string(),
            agent_label_offset(height_cm),
            format!("label:agent:{agent_id}"),
        );
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

fn location_radius_m(radius_cm: i64) -> f32 {
    (radius_cm.max(1) as f32 / 100.0).clamp(LOCATION_RADIUS_MIN_M, LOCATION_RADIUS_MAX_M)
}

fn location_label_offset(radius_m: f32) -> f32 {
    (radius_m + 0.5).max(LOCATION_LABEL_OFFSET)
}

fn agent_height_cm(height_cm: Option<i64>) -> i64 {
    height_cm.unwrap_or(agent_world::models::DEFAULT_AGENT_HEIGHT_CM)
}

fn agent_radius_m(height_cm: i64) -> f32 {
    let height_m = (agent_height_cm(Some(height_cm)) as f32 / 100.0)
        .clamp(AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M);
    (height_m * 0.35).clamp(0.08, 1.5)
}

fn agent_label_offset(height_cm: i64) -> f32 {
    let height_m = (agent_height_cm(Some(height_cm)) as f32 / 100.0)
        .clamp(AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M);
    (height_m * 0.65).max(AGENT_LABEL_OFFSET)
}

fn id_hash_fraction(id: &str) -> f32 {
    let hash = id.bytes().fold(0u32, |acc, value| {
        acc.wrapping_mul(31).wrapping_add(value as u32)
    });
    (hash % 1024) as f32 / 1024.0
}

fn asset_translation(base: Vec3, asset_id: &str) -> Vec3 {
    let angle = id_hash_fraction(asset_id) * std::f32::consts::TAU;
    let lateral = Vec3::new(angle.cos(), 0.0, angle.sin()) * ASSET_MARKER_RING_RADIUS;
    base + lateral + Vec3::Y * ASSET_MARKER_VERTICAL_OFFSET
}

fn module_visual_anchor_pos_in_snapshot(
    snapshot: &WorldSnapshot,
    anchor: &ModuleVisualAnchor,
) -> Option<GeoPos> {
    match anchor {
        ModuleVisualAnchor::Agent { agent_id } => {
            snapshot.model.agents.get(agent_id).map(|agent| agent.pos)
        }
        ModuleVisualAnchor::Location { location_id } => snapshot
            .model
            .locations
            .get(location_id)
            .map(|location| location.pos),
        ModuleVisualAnchor::Absolute { pos } => Some(*pos),
    }
}

fn module_visual_anchor_pos_in_scene(
    scene: &Viewer3dScene,
    anchor: &ModuleVisualAnchor,
) -> Option<GeoPos> {
    match anchor {
        ModuleVisualAnchor::Agent { agent_id } => scene.agent_positions.get(agent_id).copied(),
        ModuleVisualAnchor::Location { location_id } => {
            scene.location_positions.get(location_id).copied()
        }
        ModuleVisualAnchor::Absolute { pos } => Some(*pos),
    }
}

fn module_visual_translation(base: Vec3, module_id: &str, entity_id: &str) -> Vec3 {
    let hash_key = format!("{module_id}:{entity_id}");
    let angle = id_hash_fraction(hash_key.as_str()) * std::f32::consts::TAU;
    let lateral = Vec3::new(angle.cos(), 0.0, angle.sin()) * MODULE_VISUAL_RING_RADIUS;
    base + lateral + Vec3::Y * MODULE_VISUAL_VERTICAL_OFFSET
}

pub(super) fn spawn_module_visual_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    module_entity: &ModuleVisualEntity,
    anchor_pos: GeoPos,
) {
    let translation = module_visual_translation(
        geo_to_vec3(anchor_pos, origin, config.effective_cm_to_unit()),
        module_entity.module_id.as_str(),
        module_entity.entity_id.as_str(),
    );

    if let Some(entity) = scene
        .module_visual_entities
        .remove(module_entity.entity_id.as_str())
    {
        commands.entity(entity).despawn();
    }

    let visual_id = module_entity.entity_id.clone();
    let visual_label = module_entity.resolved_label();
    let visual_name = format!(
        "module_visual:{}:{}:{}",
        module_entity.module_id, module_entity.kind, module_entity.entity_id
    );

    let entity = commands
        .spawn((
            Mesh3d(assets.asset_mesh.clone()),
            MeshMaterial3d(assets.asset_material.clone()),
            Transform::from_translation(translation).with_scale(Vec3::splat(0.9)),
            Name::new(visual_name),
            AssetMarker {
                id: visual_id.clone(),
            },
            BaseScale(Vec3::splat(0.9)),
        ))
        .id();

    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            visual_label,
            AGENT_LABEL_OFFSET,
            format!("label:module_visual:{visual_id}"),
        );
    });

    scene.module_visual_entities.insert(visual_id, entity);
}

pub(super) fn spawn_power_plant_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    facility_id: &str,
    location_id: &str,
    location_pos: GeoPos,
) {
    let base = geo_to_vec3(location_pos, origin, config.effective_cm_to_unit());
    let translation = base
        + Vec3::new(
            FACILITY_MARKER_LATERAL_OFFSET,
            FACILITY_MARKER_VERTICAL_OFFSET,
            0.0,
        );

    if let Some(entity) = scene.power_plant_entities.get(facility_id) {
        commands
            .entity(*entity)
            .insert(Transform::from_translation(translation));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.power_plant_mesh.clone()),
            MeshMaterial3d(assets.power_plant_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("power_plant:{facility_id}:{location_id}")),
            PowerPlantMarker {
                id: facility_id.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            format!("plant:{facility_id}"),
            LOCATION_LABEL_OFFSET,
            format!("label:power_plant:{facility_id}"),
        );
    });
    scene
        .power_plant_entities
        .insert(facility_id.to_string(), entity);
}

pub(super) fn spawn_power_storage_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    facility_id: &str,
    location_id: &str,
    location_pos: GeoPos,
) {
    let base = geo_to_vec3(location_pos, origin, config.effective_cm_to_unit());
    let translation = base
        + Vec3::new(
            0.0,
            FACILITY_MARKER_VERTICAL_OFFSET,
            FACILITY_MARKER_LATERAL_OFFSET,
        );

    if let Some(entity) = scene.power_storage_entities.get(facility_id) {
        commands
            .entity(*entity)
            .insert(Transform::from_translation(translation));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.power_storage_mesh.clone()),
            MeshMaterial3d(assets.power_storage_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("power_storage:{facility_id}:{location_id}")),
            PowerStorageMarker {
                id: facility_id.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            format!("storage:{facility_id}"),
            LOCATION_LABEL_OFFSET,
            format!("label:power_storage:{facility_id}"),
        );
    });
    scene
        .power_storage_entities
        .insert(facility_id.to_string(), entity);
}

pub(super) fn spawn_asset_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    asset_id: &str,
    owner_pos: GeoPos,
) {
    let base = geo_to_vec3(owner_pos, origin, config.effective_cm_to_unit());
    let translation = asset_translation(base, asset_id);

    if let Some(entity) = scene.asset_entities.get(asset_id) {
        commands
            .entity(*entity)
            .insert(Transform::from_translation(translation));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.asset_mesh.clone()),
            MeshMaterial3d(assets.asset_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("asset:{asset_id}")),
            AssetMarker {
                id: asset_id.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            format!("asset:{asset_id}"),
            AGENT_LABEL_OFFSET,
            format!("label:asset:{asset_id}"),
        );
    });
    scene.asset_entities.insert(asset_id.to_string(), entity);
}

fn chunk_coord_id(coord: ChunkCoord) -> String {
    format!("{},{},{}", coord.x, coord.y, coord.z)
}

fn chunk_state_name(state: ChunkState) -> String {
    match state {
        ChunkState::Unexplored => "unexplored".to_string(),
        ChunkState::Generated => "generated".to_string(),
        ChunkState::Exhausted => "exhausted".to_string(),
    }
}

fn chunk_material(assets: &Viewer3dAssets, state: ChunkState) -> Handle<StandardMaterial> {
    match state {
        ChunkState::Unexplored => assets.chunk_unexplored_material.clone(),
        ChunkState::Generated => assets.chunk_generated_material.clone(),
        ChunkState::Exhausted => assets.chunk_exhausted_material.clone(),
    }
}

fn chunk_transform(
    coord: ChunkCoord,
    space: &SpaceConfig,
    origin: GeoPos,
    cm_to_unit: f32,
) -> Option<(Vec3, Vec3)> {
    let bounds = chunk_bounds(coord, space)?;
    let center = GeoPos::new(
        (bounds.min.x_cm + bounds.max.x_cm) * 0.5,
        (bounds.min.y_cm + bounds.max.y_cm) * 0.5,
        (bounds.min.z_cm + bounds.max.z_cm) * 0.5,
    );

    let full_size = Vec3::new(
        ((bounds.max.x_cm - bounds.min.x_cm) * cm_to_unit as f64) as f32,
        ((bounds.max.z_cm - bounds.min.z_cm) * cm_to_unit as f64) as f32,
        ((bounds.max.y_cm - bounds.min.y_cm) * cm_to_unit as f64) as f32,
    );

    let marker_scale = Vec3::new(
        (full_size.x * 0.18).clamp(CHUNK_MARKER_MIN_SIZE, CHUNK_MARKER_MAX_SIZE),
        (full_size.y * 0.08).clamp(CHUNK_MARKER_MIN_SIZE, CHUNK_MARKER_MAX_SIZE),
        (full_size.z * 0.18).clamp(CHUNK_MARKER_MIN_SIZE, CHUNK_MARKER_MAX_SIZE),
    );

    let translation = geo_to_vec3(center, origin, cm_to_unit)
        + Vec3::Y * (full_size.y * 0.5 + CHUNK_MARKER_VERTICAL_OFFSET);
    Some((translation, marker_scale))
}

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
    let Some((translation, marker_scale)) =
        chunk_transform(coord, space, origin, config.effective_cm_to_unit())
    else {
        return;
    };
    let chunk_id = chunk_coord_id(coord);
    let state_name = chunk_state_name(state);

    if let Some(entity) = scene.chunk_entities.get(&chunk_id) {
        commands.entity(*entity).insert((
            MeshMaterial3d(chunk_material(assets, state)),
            Transform::from_translation(translation).with_scale(marker_scale),
            ChunkMarker {
                id: chunk_id.clone(),
                state: state_name.clone(),
            },
            BaseScale(marker_scale),
        ));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.chunk_mesh.clone()),
            MeshMaterial3d(chunk_material(assets, state)),
            Transform::from_translation(translation).with_scale(marker_scale),
            Name::new(format!("chunk:{}:{}:{}", coord.x, coord.y, coord.z)),
            ChunkMarker {
                id: chunk_id.clone(),
                state: state_name.clone(),
            },
            BaseScale(marker_scale),
        ))
        .id();

    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            format!("chunk {chunk_id}"),
            LOCATION_LABEL_OFFSET,
            format!("label:chunk:{chunk_id}"),
        );
    });

    scene.chunk_entities.insert(chunk_id, entity);
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

pub(super) fn reset_entity_scale(
    transforms: &mut Query<(&mut Transform, Option<&BaseScale>)>,
    entity: Entity,
) {
    if let Ok((mut transform, base)) = transforms.get_mut(entity) {
        let base_scale = base.map(|scale| scale.0).unwrap_or(Vec3::ONE);
        transform.scale = base_scale;
    }
}
