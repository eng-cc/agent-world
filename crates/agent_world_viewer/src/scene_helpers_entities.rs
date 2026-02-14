use super::*;

#[derive(Clone, Copy)]
struct LocationDetailProfile {
    ring_segments: usize,
    halo_segments: usize,
    halo_radius_local: f32,
    halo_scale_local: f32,
}

fn location_detail_profile(
    radius_cm: i64,
    radiation_emission_per_tick: i64,
) -> LocationDetailProfile {
    let radius_m = location_radius_m(radius_cm);
    let ring_segments = if radius_m < 20.0 {
        3
    } else if radius_m < 120.0 {
        4
    } else if radius_m < 500.0 {
        5
    } else {
        6
    };

    let radiation_ratio = location_radiation_ratio(radiation_emission_per_tick);
    let halo_segments = if radiation_emission_per_tick <= 0 {
        0
    } else {
        (3 + (radiation_ratio * 4.0).round() as usize).clamp(3, 7)
    };

    LocationDetailProfile {
        ring_segments,
        halo_segments,
        halo_radius_local: 1.24 + radiation_ratio * 0.36,
        halo_scale_local: 0.03 + radiation_ratio * 0.03,
    }
}

fn location_ring_local_scale(radius_cm: i64) -> Vec3 {
    let radius_m = location_radius_m(radius_cm);
    let width = if radius_m < 10.0 {
        0.18
    } else if radius_m < 80.0 {
        0.12
    } else if radius_m < 400.0 {
        0.09
    } else {
        0.06
    };
    Vec3::new(width, width * 0.6, width * 0.8)
}

pub(super) fn spawn_location_detail_children(
    parent: &mut ChildSpawnerCommands,
    assets: &Viewer3dAssets,
    location_id: &str,
    material: MaterialKind,
    radius_cm: i64,
    radiation_emission_per_tick: i64,
) {
    let detail = location_detail_profile(radius_cm, radiation_emission_per_tick);
    let phase = id_hash_fraction(location_id) * std::f32::consts::TAU;
    let ring_scale = location_ring_local_scale(radius_cm);
    let ring_material = assets.location_material_library.handle_for(material);

    for idx in 0..detail.ring_segments {
        let angle = phase + (idx as f32 / detail.ring_segments as f32) * std::f32::consts::TAU;
        let local_radius = if idx % 2 == 0 {
            LOCATION_DETAIL_RING_RADIUS_BASE
        } else {
            LOCATION_DETAIL_RING_RADIUS_ALT
        };
        let y_band = match idx % 3 {
            0 => -LOCATION_DETAIL_RING_Y_BAND,
            1 => 0.0,
            _ => LOCATION_DETAIL_RING_Y_BAND,
        };
        parent.spawn((
            Mesh3d(assets.agent_module_marker_mesh.clone()),
            MeshMaterial3d(ring_material.clone()),
            Transform::from_translation(Vec3::new(
                angle.cos() * local_radius,
                y_band,
                angle.sin() * local_radius,
            ))
            .with_rotation(Quat::from_rotation_y(angle))
            .with_scale(ring_scale),
            Name::new(format!("location:detail:ring:{location_id}:{idx}")),
        ));
    }

    if detail.halo_segments == 0 {
        return;
    }

    for idx in 0..detail.halo_segments {
        let angle =
            phase * 0.5 + (idx as f32 / detail.halo_segments as f32) * std::f32::consts::TAU;
        let radius = detail.halo_radius_local
            + if idx % 2 == 0 {
                0.0
            } else {
                LOCATION_DETAIL_HALO_RADIUS_JITTER
            };
        let y_offset = if idx % 2 == 0 {
            LOCATION_DETAIL_HALO_Y_OFFSET
        } else {
            -LOCATION_DETAIL_HALO_Y_OFFSET
        };
        let scale = Vec3::splat(detail.halo_scale_local * if idx % 2 == 0 { 1.0 } else { 0.82 });
        parent.spawn((
            Mesh3d(assets.location_mesh.clone()),
            MeshMaterial3d(assets.agent_module_marker_material.clone()),
            Transform::from_translation(Vec3::new(
                angle.cos() * radius,
                y_offset,
                angle.sin() * radius,
            ))
            .with_scale(scale),
            Name::new(format!("location:detail:halo:{location_id}:{idx}")),
        ));
    }
}

pub(super) fn location_label_offset(radius_m: f32) -> f32 {
    (radius_m + 0.5).max(LOCATION_LABEL_OFFSET)
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
