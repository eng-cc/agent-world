use super::*;

pub(super) fn setup_3d_scene(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    external_mesh: Res<ViewerExternalMeshConfig>,
    external_material: Res<ViewerExternalMaterialConfig>,
    external_texture: Res<ViewerExternalTextureConfig>,
    variant_preview: Res<MaterialVariantPreviewState>,
    camera_mode: Res<ViewerCameraMode>,
    mut scene: ResMut<Viewer3dScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let root_entity = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            Viewer3dSceneRoot,
        ))
        .id();
    scene.root_entity = Some(root_entity);

    let label_font = load_runtime_cjk_font(&asset_server);
    let resolved_theme_assets = resolve_theme_scene_assets(
        &config,
        &external_mesh,
        &external_material,
        &external_texture,
        &variant_preview,
        &mut meshes,
        &asset_server,
    );
    let agent_mesh = resolved_theme_assets.agent_mesh.clone();
    let location_mesh = resolved_theme_assets.location_mesh.clone();
    let asset_mesh = resolved_theme_assets.asset_mesh.clone();
    let power_plant_mesh = resolved_theme_assets.power_plant_mesh.clone();
    let world_box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let agent_material = materials.add(resolved_theme_assets.agent_material.clone());
    let asset_material = materials.add(resolved_theme_assets.asset_material.clone());
    let power_plant_material = materials.add(resolved_theme_assets.power_plant_material.clone());
    let agent_module_marker_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let module_marker_color = config
        .visual
        .agent_variant_palette
        .map(|palette| palette[0])
        .unwrap_or([0.16, 0.92, 0.98]);
    let agent_module_marker_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(module_marker_color),
        unlit: true,
        ..default()
    });
    let fragment_element_material_library =
        build_fragment_element_material_handles(&mut materials, config.materials.fragment);
    let chunk_unexplored_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.30, 0.42, 0.66, 0.22),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let chunk_generated_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.24, 0.78, 0.44, 0.30),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let chunk_exhausted_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.62, 0.40, 0.28, 0.30),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let world_floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.11),
        unlit: true,
        ..default()
    });
    let world_bounds_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.48, 0.65, 0.10),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let (
        location_core_silicate_material,
        location_core_metal_material,
        location_core_ice_material,
        location_halo_material,
    ) = if let Some(location_override) = resolved_theme_assets.location_override_materials {
        (
            materials.add(location_override.core_silicate),
            materials.add(location_override.core_metal),
            materials.add(location_override.core_ice),
            materials.add(location_override.halo),
        )
    } else {
        (
            chunk_unexplored_material.clone(),
            chunk_generated_material.clone(),
            chunk_exhausted_material.clone(),
            world_bounds_material.clone(),
        )
    };
    let world_grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.29, 0.36, 0.48, 0.42),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let heat_low_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.24, 0.52, 0.88, 0.42),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let heat_mid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.62, 0.18, 0.48),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let heat_high_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.26, 0.20, 0.55),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let flow_power_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.72, 0.98, 0.62),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let flow_trade_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.80, 0.26, 0.58),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(Viewer3dAssets {
        agent_mesh,
        agent_material,
        agent_module_marker_mesh,
        agent_module_marker_material,
        location_mesh,
        fragment_element_material_library,
        asset_mesh,
        asset_material,
        power_plant_mesh,
        power_plant_material,
        location_core_silicate_material,
        location_core_metal_material,
        location_core_ice_material,
        location_halo_material,
        chunk_unexplored_material,
        chunk_generated_material,
        chunk_exhausted_material,
        world_box_mesh,
        world_floor_material,
        world_bounds_material,
        world_grid_material,
        heat_low_material,
        heat_mid_material,
        heat_high_material,
        flow_power_material,
        flow_trade_material,
        label_font,
    });

    let mode = *camera_mode;
    let orbit = camera_orbit_preset(mode, None, config.effective_cm_to_unit());
    let mut transform = Transform::default();
    orbit.apply_to_transform(&mut transform);
    let mut projection = camera_projection_for_mode(mode, &config);
    if mode == ViewerCameraMode::TwoD {
        sync_2d_zoom_projection(&mut projection, orbit.radius, config.effective_cm_to_unit());
    }
    let (tonemapping, deband_dither, color_grading, bloom) =
        camera_post_process_components(&config);
    let mut camera_entity = commands.spawn((
        Camera3d::default(),
        projection,
        Camera {
            order: 0,
            ..default()
        },
        transform,
        Viewer3dCamera,
        orbit,
        tonemapping,
        deband_dither,
        color_grading,
    ));
    if let Some(settings) = bloom {
        camera_entity.insert(settings);
    }

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.94, 0.97, 1.0),
        brightness: config.lighting.ambient_brightness,
        affects_lightmapped_meshes: true,
    });

    let (key_illuminance, fill_illuminance, rim_illuminance) =
        lighting_illuminance_triplet(&config);

    commands.spawn((
        DirectionalLight {
            illuminance: key_illuminance,
            shadows_enabled: config.lighting.shadows_enabled,
            ..default()
        },
        Transform::from_xyz(24.0, 36.0, 22.0).looking_at(Vec3::ZERO, Vec3::Y),
        ViewerLightRigRole::Key,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: fill_illuminance,
            color: Color::srgb(0.74, 0.82, 0.92),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-18.0, 20.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
        ViewerLightRigRole::Fill,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: rim_illuminance,
            color: Color::srgb(0.96, 0.88, 0.78),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(28.0, 16.0, -24.0).looking_at(Vec3::ZERO, Vec3::Y),
        ViewerLightRigRole::Rim,
    ));
}

pub(super) fn handle_material_variant_preview_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut preview_state: ResMut<MaterialVariantPreviewState>,
    config: Res<Viewer3dConfig>,
    assets: Option<Res<Viewer3dAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !keyboard.just_pressed(KeyCode::F8) {
        return;
    }

    preview_state.active = preview_state.active.next();
    let Some(assets) = assets else {
        return;
    };
    apply_material_variant_to_scene_materials(
        &mut materials,
        &assets,
        &config,
        preview_state.active,
    );
}

pub(super) fn update_3d_scene(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
    mut selection: ResMut<ViewerSelection>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>)>,
    state: Res<ViewerState>,
) {
    let Some(snapshot) = state.snapshot.as_ref() else {
        return;
    };

    let snapshot_time = snapshot.time;
    if scene_requires_full_rebuild(&scene, snapshot) {
        rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, snapshot);
        scene.last_event_id = None;
        selection.clear();
    } else {
        refresh_scene_dirty_objects(&mut commands, &config, &assets, &mut scene, snapshot);
    }
    scene.last_snapshot_time = Some(snapshot_time);

    apply_events_to_scene(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        snapshot,
        snapshot_time,
        &state.events,
    );

    if config.highlight_selected {
        if let Some(current) = selection.current.as_ref() {
            if should_apply_scale_highlight(current.kind) {
                apply_entity_highlight(&mut transforms, current.entity);
            } else {
                reset_entity_scale(&mut transforms, current.entity);
            }
        }
    } else if let Some(current) = selection.current.as_ref() {
        reset_entity_scale(&mut transforms, current.entity);
    }
}

pub(super) fn update_3d_viewport(mut cameras: Query<&mut Camera, With<Viewer3dCamera>>) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    camera.viewport = None;
}
