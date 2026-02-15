use super::*;

const TWO_D_AGENT_MARKER_MIN_RADIUS_WORLD: f32 = 0.0003;
const TWO_D_AGENT_MARKER_MIN_THICKNESS_WORLD: f32 = 0.00004;
const TWO_D_AGENT_MARKER_MAX_THICKNESS_WORLD: f32 = 0.0016;
const TWO_D_AGENT_MARKER_MIN_LIFT_M: f32 = 0.18;
const TWO_D_AGENT_MARKER_MAX_LIFT_M: f32 = 0.85;

pub(super) fn spawn_agent_two_d_map_marker(
    parent: &mut ChildSpawnerCommands,
    assets: &Viewer3dAssets,
    agent_id: &str,
    height_cm: i64,
    module_count: usize,
    cm_to_unit: f32,
) {
    let (world_radius, thickness, y) = two_d_agent_marker_profile(height_cm, cm_to_unit);

    let base_scale = Vec3::new(world_radius * 2.0, thickness, world_radius * 2.0);
    parent.spawn((
        Mesh3d(assets.agent_module_marker_mesh.clone()),
        MeshMaterial3d(assets.agent_module_marker_material.clone()),
        Transform::from_translation(Vec3::new(0.0, y, 0.0)).with_scale(base_scale),
        BaseScale(base_scale),
        Visibility::Visible,
        Name::new(format!("map2d:agent:plate:{agent_id}")),
        TwoDMapMarker,
    ));

    let module_ratio =
        module_count.min(AGENT_MODULE_MARKER_MAX) as f32 / AGENT_MODULE_MARKER_MAX as f32;
    if module_ratio > 0.0 {
        let outer_radius = world_radius * (1.10 + module_ratio * 0.45);
        let outer_scale = Vec3::new(outer_radius * 2.0, thickness * 0.55, outer_radius * 2.0);
        parent.spawn((
            Mesh3d(assets.agent_module_marker_mesh.clone()),
            MeshMaterial3d(assets.chunk_generated_material.clone()),
            Transform::from_translation(Vec3::new(0.0, y - thickness * 0.45, 0.0))
                .with_scale(outer_scale),
            BaseScale(outer_scale),
            Visibility::Visible,
            Name::new(format!("map2d:agent:module_band:{agent_id}")),
            TwoDMapMarker,
        ));
    }

    parent.spawn((
        Mesh3d(assets.location_mesh.clone()),
        MeshMaterial3d(assets.agent_material.clone()),
        Transform::from_translation(Vec3::new(0.0, y + thickness * 0.65, 0.0))
            .with_scale(Vec3::splat(world_radius * 0.58)),
        BaseScale(Vec3::splat(world_radius * 0.58)),
        Visibility::Visible,
        Name::new(format!("map2d:agent:center:{agent_id}")),
        TwoDMapMarker,
    ));
}

fn two_d_agent_marker_profile(height_cm: i64, cm_to_unit: f32) -> (f32, f32, f32) {
    let agent_height_m = (agent_height_cm(Some(height_cm)) as f32 / 100.0)
        .clamp(AGENT_HEIGHT_MIN_M, AGENT_HEIGHT_MAX_M);
    let units_per_m = world_units_per_meter(cm_to_unit);
    let physical_radius = (agent_height_m * 0.62).clamp(0.38, 0.95) * units_per_m;
    let world_radius = physical_radius.max(TWO_D_AGENT_MARKER_MIN_RADIUS_WORLD);
    let thickness = (world_radius * 0.18).clamp(
        TWO_D_AGENT_MARKER_MIN_THICKNESS_WORLD,
        TWO_D_AGENT_MARKER_MAX_THICKNESS_WORLD,
    );
    let y = (agent_height_m * 0.35)
        .clamp(TWO_D_AGENT_MARKER_MIN_LIFT_M, TWO_D_AGENT_MARKER_MAX_LIFT_M)
        * units_per_m
        + thickness * 0.5;
    (world_radius, thickness, y)
}

pub(super) fn id_hash_fraction(id: &str) -> f32 {
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

pub(super) fn module_visual_anchor_pos_in_snapshot(
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

pub(super) fn module_visual_anchor_pos_in_scene(
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
    attach_to_scene_root(commands, scene, entity);

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
    attach_to_scene_root(commands, scene, entity);
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
    attach_to_scene_root(commands, scene, entity);
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
    attach_to_scene_root(commands, scene, entity);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_d_agent_marker_profile_enforces_readable_radius() {
        let (radius, thickness, y) = two_d_agent_marker_profile(100, 0.00001);
        assert!(radius >= TWO_D_AGENT_MARKER_MIN_RADIUS_WORLD);
        assert!(thickness >= TWO_D_AGENT_MARKER_MIN_THICKNESS_WORLD);
        assert!(thickness <= TWO_D_AGENT_MARKER_MAX_THICKNESS_WORLD);
        assert!(y > 0.0);
    }
}

fn spawn_chunk_line_segments(
    commands: &mut Commands,
    assets: &Viewer3dAssets,
    scene: &Viewer3dScene,
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
    y: f32,
    chunk_id: &str,
    state_name: &str,
    state: ChunkState,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let thickness = grid_line_thickness(GridLineKind::Chunk, ViewerCameraMode::TwoD);

    let x_span = max_z - min_z;
    let x_line_scale = grid_line_scale(GridLineAxis::AlongZ, x_span, thickness);
    for (idx, x) in [min_x, max_x].into_iter().enumerate() {
        let entity = commands
            .spawn((
                Mesh3d(assets.world_box_mesh.clone()),
                MeshMaterial3d(chunk_material(assets, state)),
                Transform::from_translation(Vec3::new(x, y, (min_z + max_z) * 0.5))
                    .with_scale(x_line_scale),
                Name::new(format!("chunk:grid:x:{chunk_id}:{idx}")),
                ChunkMarker {
                    id: chunk_id.to_string(),
                    state: state_name.to_string(),
                    min_x,
                    max_x,
                    min_z,
                    max_z,
                    pick_y: y,
                },
                BaseScale(x_line_scale),
                GridLineVisual {
                    kind: GridLineKind::Chunk,
                    axis: GridLineAxis::AlongZ,
                    span: x_span,
                },
            ))
            .id();
        attach_to_scene_root(commands, scene, entity);
        entities.push(entity);
    }

    let z_span = max_x - min_x;
    let z_line_scale = grid_line_scale(GridLineAxis::AlongX, z_span, thickness);
    for (idx, z) in [min_z, max_z].into_iter().enumerate() {
        let entity = commands
            .spawn((
                Mesh3d(assets.world_box_mesh.clone()),
                MeshMaterial3d(chunk_material(assets, state)),
                Transform::from_translation(Vec3::new((min_x + max_x) * 0.5, y, z))
                    .with_scale(z_line_scale),
                Name::new(format!("chunk:grid:z:{chunk_id}:{idx}")),
                ChunkMarker {
                    id: chunk_id.to_string(),
                    state: state_name.to_string(),
                    min_x,
                    max_x,
                    min_z,
                    max_z,
                    pick_y: y,
                },
                BaseScale(z_line_scale),
                GridLineVisual {
                    kind: GridLineKind::Chunk,
                    axis: GridLineAxis::AlongX,
                    span: z_span,
                },
            ))
            .id();
        attach_to_scene_root(commands, scene, entity);
        entities.push(entity);
    }

    entities
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
    let Some(bounds) = chunk_bounds(coord, space) else {
        return;
    };
    let cm_to_unit = config.effective_cm_to_unit();
    let chunk_id = chunk_coord_id(coord);
    let state_name = chunk_state_name(state);

    if let Some(lines) = scene.chunk_line_entities.remove(&chunk_id) {
        for entity in lines {
            commands.entity(entity).despawn();
        }
    }
    scene.chunk_entities.remove(&chunk_id);

    let min_x = ((bounds.min.x_cm - origin.x_cm) * cm_to_unit as f64) as f32;
    let max_x = ((bounds.max.x_cm - origin.x_cm) * cm_to_unit as f64) as f32;
    let min_z = ((bounds.min.y_cm - origin.y_cm) * cm_to_unit as f64) as f32;
    let max_z = ((bounds.max.y_cm - origin.y_cm) * cm_to_unit as f64) as f32;
    let thickness = grid_line_thickness(GridLineKind::Chunk, ViewerCameraMode::TwoD);
    let y = -((space.height_cm as f32) * cm_to_unit * 0.5) + thickness * 0.7;

    let lines = spawn_chunk_line_segments(
        commands,
        assets,
        scene,
        min_x,
        max_x,
        min_z,
        max_z,
        y,
        &chunk_id,
        &state_name,
        state,
    );

    if let Some(anchor) = lines.first().copied() {
        commands.entity(anchor).with_children(|parent| {
            spawn_label(
                parent,
                assets,
                format!("chunk {chunk_id}"),
                LOCATION_LABEL_OFFSET,
                format!("label:chunk:{chunk_id}"),
            );
        });
        scene.chunk_entities.insert(chunk_id.clone(), anchor);
    }

    scene.chunk_line_entities.insert(chunk_id, lines);
}
