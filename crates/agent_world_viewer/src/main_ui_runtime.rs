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
    mut fonts: ResMut<Assets<Font>>,
    asset_server: Res<AssetServer>,
) {
    let geometry_tier = config.assets.geometry_tier;
    let root_entity = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            Viewer3dSceneRoot,
        ))
        .id();
    scene.root_entity = Some(root_entity);

    let label_font = load_embedded_cjk_font(&mut fonts);
    let agent_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.agent_mesh_asset.as_deref(),
        || Capsule3d::new(AGENT_BODY_MESH_RADIUS, AGENT_BODY_MESH_LENGTH).into(),
    );
    let agent_module_marker_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let location_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.location_mesh_asset.as_deref(),
        || location_mesh_for_geometry_tier(geometry_tier),
    );
    let asset_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.asset_mesh_asset.as_deref(),
        || asset_mesh_for_geometry_tier(geometry_tier),
    );
    let power_plant_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.power_plant_mesh_asset.as_deref(),
        || power_plant_mesh_for_geometry_tier(geometry_tier),
    );
    let power_storage_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.power_storage_mesh_asset.as_deref(),
        || power_storage_mesh_for_geometry_tier(geometry_tier),
    );
    let world_box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let agent_texture = resolve_texture_slot(&asset_server, &external_texture.agent);
    let location_texture = resolve_texture_slot(&asset_server, &external_texture.location);
    let asset_texture = resolve_texture_slot(&asset_server, &external_texture.asset);
    let power_plant_texture = resolve_texture_slot(&asset_server, &external_texture.power_plant);
    let power_storage_texture =
        resolve_texture_slot(&asset_server, &external_texture.power_storage);
    let variant_scalars = material_variant_scalars(variant_preview.active);
    let agent_roughness = apply_material_variant_scalar(
        config.materials.agent.roughness,
        variant_scalars.roughness_scale,
    );
    let agent_metallic = apply_material_variant_scalar(
        config.materials.agent.metallic,
        variant_scalars.metallic_scale,
    );
    let asset_roughness = apply_material_variant_scalar(
        config.materials.asset.roughness,
        variant_scalars.roughness_scale,
    );
    let asset_metallic = apply_material_variant_scalar(
        config.materials.asset.metallic,
        variant_scalars.metallic_scale,
    );
    let facility_roughness = apply_material_variant_scalar(
        config.materials.facility.roughness,
        variant_scalars.roughness_scale,
    );
    let facility_metallic = apply_material_variant_scalar(
        config.materials.facility.metallic,
        variant_scalars.metallic_scale,
    );
    let agent_base_color =
        resolve_srgb_slot_color([1.0, 0.42, 0.22], external_material.agent.base_color_srgb);
    let agent_emissive_color = resolve_srgb_slot_color(
        [0.90, 0.38, 0.20],
        external_material.agent.emissive_color_srgb,
    );
    let agent_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(agent_base_color),
        base_color_texture: agent_texture.base_color_texture,
        normal_map_texture: agent_texture.normal_map_texture,
        metallic_roughness_texture: agent_texture.metallic_roughness_texture,
        emissive_texture: agent_texture.emissive_texture,
        perceptual_roughness: agent_roughness,
        metallic: agent_metallic,
        emissive: emissive_from_srgb_with_boost(
            agent_emissive_color,
            config.materials.agent.emissive_boost,
        ),
        ..default()
    });
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
    let asset_base_color =
        resolve_srgb_slot_color([0.82, 0.76, 0.34], external_material.asset.base_color_srgb);
    let asset_emissive_color = resolve_srgb_slot_color(
        [0.82, 0.76, 0.34],
        external_material.asset.emissive_color_srgb,
    );
    let asset_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(asset_base_color),
        base_color_texture: asset_texture.base_color_texture,
        normal_map_texture: asset_texture.normal_map_texture,
        metallic_roughness_texture: asset_texture.metallic_roughness_texture,
        emissive_texture: asset_texture.emissive_texture,
        perceptual_roughness: asset_roughness,
        metallic: asset_metallic,
        emissive: emissive_from_srgb_with_boost(
            asset_emissive_color,
            config.materials.asset.emissive_boost,
        ),
        ..default()
    });
    let power_plant_base_color = resolve_srgb_slot_color(
        [0.95, 0.42, 0.20],
        external_material.power_plant.base_color_srgb,
    );
    let power_plant_emissive_color = resolve_srgb_slot_color(
        [0.95, 0.42, 0.20],
        external_material.power_plant.emissive_color_srgb,
    );
    let power_plant_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(power_plant_base_color),
        base_color_texture: power_plant_texture.base_color_texture,
        normal_map_texture: power_plant_texture.normal_map_texture,
        metallic_roughness_texture: power_plant_texture.metallic_roughness_texture,
        emissive_texture: power_plant_texture.emissive_texture,
        perceptual_roughness: facility_roughness,
        metallic: facility_metallic,
        emissive: emissive_from_srgb_with_boost(
            power_plant_emissive_color,
            config.materials.facility.emissive_boost,
        ),
        ..default()
    });
    let power_storage_base_color = resolve_srgb_slot_color(
        [0.20, 0.86, 0.48],
        external_material.power_storage.base_color_srgb,
    );
    let power_storage_emissive_color = resolve_srgb_slot_color(
        [0.20, 0.86, 0.48],
        external_material.power_storage.emissive_color_srgb,
    );
    let power_storage_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(power_storage_base_color),
        base_color_texture: power_storage_texture.base_color_texture,
        normal_map_texture: power_storage_texture.normal_map_texture,
        metallic_roughness_texture: power_storage_texture.metallic_roughness_texture,
        emissive_texture: power_storage_texture.emissive_texture,
        perceptual_roughness: facility_roughness,
        metallic: facility_metallic,
        emissive: emissive_from_srgb_with_boost(
            power_storage_emissive_color,
            config.materials.facility.emissive_boost,
        ),
        ..default()
    });
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
    ) = if location_style_override_enabled(external_material.location, &external_texture.location) {
        let location_base_color = resolve_srgb_slot_color(
            [0.30, 0.42, 0.66],
            external_material.location.base_color_srgb,
        );
        let location_emissive_color = resolve_srgb_slot_color(
            location_base_color,
            external_material.location.emissive_color_srgb,
        );
        let location_texture = location_texture.clone();
        let location_core_material = |alpha: f32| StandardMaterial {
            base_color: color_from_srgb_with_alpha(location_base_color, alpha),
            base_color_texture: location_texture.base_color_texture.clone(),
            normal_map_texture: location_texture.normal_map_texture.clone(),
            metallic_roughness_texture: location_texture.metallic_roughness_texture.clone(),
            emissive_texture: location_texture.emissive_texture.clone(),
            perceptual_roughness: facility_roughness,
            metallic: facility_metallic,
            emissive: color_from_srgb(location_emissive_color).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        };

        (
            materials.add(location_core_material(0.22)),
            materials.add(location_core_material(0.30)),
            materials.add(location_core_material(0.30)),
            materials.add(StandardMaterial {
                base_color: color_from_srgb_with_alpha(location_base_color, 0.10),
                base_color_texture: location_texture.base_color_texture,
                normal_map_texture: location_texture.normal_map_texture,
                metallic_roughness_texture: location_texture.metallic_roughness_texture,
                emissive_texture: location_texture.emissive_texture,
                emissive: color_from_srgb(location_emissive_color).into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
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
        power_storage_mesh,
        power_storage_material,
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

#[allow(dead_code)]
pub(super) fn setup_ui(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    _asset_server: Res<AssetServer>,
    copyable_panel_state: Option<Res<CopyableTextPanelState>>,
) {
    let font = load_embedded_cjk_font(&mut fonts);
    let i18n = UiI18n::default();
    let locale = i18n.locale;
    let copyable_panel_visible = copyable_panel_state
        .as_ref()
        .map(|state| state.visible)
        .unwrap_or(true);

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        IsDefaultUiCamera,
    ));

    commands
        .spawn((
            Node {
                width: Val::Px(UI_PANEL_WIDTH),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::left(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.08, 0.1)),
            BorderColor::all(Color::srgb(0.18, 0.2, 0.24)),
        ))
        .with_children(|root| {
            spawn_top_panel_toggle(root, font.clone(), locale, copyable_panel_visible);

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(372.0),
                    min_height: Val::Px(260.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    overflow: Overflow::scroll_y(),
                    flex_shrink: 0.0,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.11, 0.14)),
                BorderColor::all(Color::srgb(0.2, 0.22, 0.26)),
                ScrollPosition::default(),
                TopPanelContainer,
                TopPanelScroll,
            ))
            .with_children(|bar| {
                bar.spawn(Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(32.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(6.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|controls| {
                    for (label, control) in [
                        (ViewerControl::Play, ViewerControl::Play),
                        (ViewerControl::Pause, ViewerControl::Pause),
                        (
                            ViewerControl::Step { count: 1 },
                            ViewerControl::Step { count: 1 },
                        ),
                        (
                            ViewerControl::Seek { tick: 0 },
                            ViewerControl::Seek { tick: 0 },
                        ),
                    ] {
                        controls
                            .spawn((
                                Button,
                                Node {
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    height: Val::Px(28.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.24)),
                                ControlButton { control },
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(control_button_label(&label, locale)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

                bar.spawn((
                    Text::new(status_line(&ConnectionStatus::Connecting, locale)),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    StatusText,
                ));

                bar.spawn((
                    Text::new(selection_line(&ViewerSelection::default(), locale)),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    SelectionText,
                ));

                spawn_world_overlay_controls(bar, font.clone(), locale);
                spawn_diagnosis_panel(bar, font.clone(), locale);
                spawn_event_object_link_controls(bar, font.clone(), locale);
                spawn_timeline_controls(bar, font.clone(), locale);
            });

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    overflow: Overflow::scroll_y(),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
                BorderColor::all(Color::srgb(0.2, 0.22, 0.26)),
                ScrollPosition::default(),
                RightPanelScroll,
            ))
            .with_children(|content| {
                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(140.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                        BorderColor::all(Color::srgb(0.24, 0.26, 0.32)),
                    ))
                    .with_children(|summary| {
                        summary.spawn((
                            Text::new(summary_no_snapshot(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            SummaryText,
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(170.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.11, 0.14)),
                        BorderColor::all(Color::srgb(0.21, 0.23, 0.29)),
                    ))
                    .with_children(|activity| {
                        activity.spawn((
                            Text::new(agents_activity_no_snapshot(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.86, 0.88, 0.92)),
                            AgentActivityText,
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(240.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.09, 0.1, 0.13)),
                        BorderColor::all(Color::srgb(0.2, 0.22, 0.28)),
                    ))
                    .with_children(|details| {
                        details.spawn((
                            Text::new(details_click_to_inspect(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.88, 0.9, 0.94)),
                            SelectionDetailsText,
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(260.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.09, 0.1, 0.12)),
                        BorderColor::all(Color::srgb(0.2, 0.22, 0.28)),
                    ))
                    .with_children(|events| {
                        events.spawn((
                            Text::new(events_empty(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                            EventsText,
                        ));

                        spawn_event_click_list(events, font.clone(), locale);
                    });
            });
        });
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

#[allow(dead_code)]
pub(super) fn update_ui(
    state: Res<ViewerState>,
    selection: Res<ViewerSelection>,
    viewer_3d_config: Option<Res<Viewer3dConfig>>,
    i18n: Option<Res<UiI18n>>,
    timeline: Option<Res<TimelineUiState>>,
    mut queries: ParamSet<(
        Query<&mut Text, With<StatusText>>,
        Query<&mut Text, With<SummaryText>>,
        Query<&mut Text, With<EventsText>>,
        Query<&mut Text, With<SelectionText>>,
        Query<&mut Text, With<AgentActivityText>>,
        Query<&mut Text, With<SelectionDetailsText>>,
    )>,
) {
    let timeline_changed = timeline
        .as_ref()
        .map(|timeline| timeline.is_changed())
        .unwrap_or(false);
    let locale_changed = i18n
        .as_ref()
        .map(|locale| locale.is_changed())
        .unwrap_or(false);
    let physical_config_changed = viewer_3d_config
        .as_ref()
        .map(|config| config.is_changed())
        .unwrap_or(false);
    if !state.is_changed()
        && !selection.is_changed()
        && !timeline_changed
        && !locale_changed
        && !physical_config_changed
    {
        return;
    }

    let locale = locale_or_default(i18n.as_deref());

    if let Ok(mut text) = queries.p0().single_mut() {
        text.0 = status_line(&state.status, locale);
    }

    if let Ok(mut text) = queries.p1().single_mut() {
        text.0 = ui_locale_text::localize_world_summary_block(
            world_summary(
                state.snapshot.as_ref(),
                state.metrics.as_ref(),
                viewer_3d_config.as_deref().map(|cfg| &cfg.physical),
            ),
            locale,
        );
    }

    let focus_tick = timeline.as_ref().and_then(|timeline| {
        if timeline.manual_override || timeline.drag_active {
            Some(timeline.target_tick)
        } else {
            None
        }
    });

    if let Ok(mut text) = queries.p2().single_mut() {
        text.0 = ui_locale_text::localize_events_summary_block(
            events_summary(&state.events, focus_tick),
            locale,
        );
    }

    if let Ok(mut text) = queries.p3().single_mut() {
        text.0 = selection_line(&selection, locale);
    }

    if let Ok(mut text) = queries.p4().single_mut() {
        text.0 = ui_locale_text::localize_agent_activity_block(
            agent_activity_summary(state.snapshot.as_ref(), &state.events),
            locale,
        );
    }

    if let Ok(mut text) = queries.p5().single_mut() {
        let reference_radiation_area_m2 = viewer_3d_config
            .as_deref()
            .map(|config| config.physical.reference_radiation_area_m2)
            .unwrap_or(1.0);
        text.0 = ui_locale_text::localize_details_block(
            selection_details_summary(
                &selection,
                state.snapshot.as_ref(),
                &state.events,
                &state.decision_traces,
                reference_radiation_area_m2,
            ),
            locale,
        );
    }
}
pub(super) fn update_3d_viewport(mut cameras: Query<&mut Camera, With<Viewer3dCamera>>) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    camera.viewport = None;
}
