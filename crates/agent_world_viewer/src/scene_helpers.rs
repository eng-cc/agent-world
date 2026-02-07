use super::*;

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
        .chain(scene.background_entities.iter())
    {
        commands.entity(*entity).despawn();
    }

    scene.agent_entities.clear();
    scene.location_entities.clear();
    scene.location_positions.clear();
    scene.background_entities.clear();

    let origin = space_origin(&snapshot.config.space);
    scene.origin = Some(origin);
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
        );
    }

    for (agent_id, agent) in snapshot.model.agents.iter() {
        spawn_agent_entity(commands, config, assets, scene, origin, agent_id, agent.pos);
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
                ..
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
                );
            }
            WorldEventKind::AgentRegistered { agent_id, pos, .. } => {
                spawn_agent_entity(commands, config, assets, scene, origin, agent_id, *pos);
            }
            WorldEventKind::AgentMoved { agent_id, to, .. } => {
                if let Some(pos) = scene.location_positions.get(to) {
                    spawn_agent_entity(commands, config, assets, scene, origin, agent_id, *pos);
                }
            }
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
    let world_width = (space.width_cm as f32 * config.cm_to_unit).max(WORLD_MIN_AXIS);
    let world_depth = (space.depth_cm as f32 * config.cm_to_unit).max(WORLD_MIN_AXIS);
    let world_height = (space.height_cm as f32 * config.cm_to_unit).max(WORLD_MIN_AXIS);

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
) {
    scene
        .location_positions
        .insert(location_id.to_string(), pos);

    if !config.show_locations {
        return;
    }

    let translation = geo_to_vec3(pos, origin, config.cm_to_unit);
    if let Some(entity) = scene.location_entities.get(location_id) {
        commands.entity(*entity).insert((
            Transform::from_translation(translation),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
            },
        ));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.location_mesh.clone()),
            MeshMaterial3d(assets.location_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("location:{location_id}:{name}")),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            format!("{name}"),
            LOCATION_LABEL_OFFSET,
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
) {
    if !config.show_agents {
        return;
    }

    let translation = geo_to_vec3(pos, origin, config.cm_to_unit);
    if let Some(entity) = scene.agent_entities.get(agent_id) {
        commands
            .entity(*entity)
            .insert(Transform::from_translation(translation));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.agent_mesh.clone()),
            MeshMaterial3d(assets.agent_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("agent:{agent_id}")),
            AgentMarker {
                id: agent_id.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            agent_id.to_string(),
            AGENT_LABEL_OFFSET,
            format!("label:agent:{agent_id}"),
        );
    });
    scene.agent_entities.insert(agent_id.to_string(), entity);
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
