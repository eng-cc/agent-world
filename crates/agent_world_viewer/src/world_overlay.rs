use agent_world::simulator::{
    ChunkState, PowerEvent, ResourceKind, ResourceOwner, WorldEvent, WorldEventKind, WorldSnapshot,
};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::i18n::{locale_or_default, UiI18n, UiLocale};
use crate::ui_locale_text::{overlay_button_label, overlay_loading, overlay_status};

use super::*;

const FLOW_WINDOW: usize = 28;
const HEAT_BASE_HEIGHT: f32 = 0.25;
const HEAT_MAX_HEIGHT: f32 = 1.8;
const HEAT_OFFSET_Y: f32 = 0.2;
const FLOW_OFFSET_Y: f32 = 0.18;
const FLOW_MIN_THICKNESS: f32 = 0.03;
const FLOW_MAX_THICKNESS: f32 = 0.12;

#[derive(Resource, Clone, Copy)]
pub(super) struct WorldOverlayConfig {
    pub show_chunk_overlay: bool,
    pub show_resource_heatmap: bool,
    pub show_flow_overlay: bool,
}

impl Default for WorldOverlayConfig {
    fn default() -> Self {
        Self {
            show_chunk_overlay: true,
            show_resource_heatmap: true,
            show_flow_overlay: true,
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct WorldOverlayUiState {
    pub status_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldOverlayKind {
    Chunk,
    Heat,
    Flow,
}

#[derive(Component)]
pub(super) struct WorldOverlayToggleButton {
    kind: WorldOverlayKind,
}

#[derive(Component)]
pub(super) struct WorldOverlayToggleLabel {
    kind: WorldOverlayKind,
}

#[derive(Component)]
pub(super) struct WorldOverlayStatusText;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlowSegmentKind {
    Power,
    Trade,
}

#[derive(Debug, Clone)]
struct FlowSegment {
    from: Vec3,
    to: Vec3,
    amount: i64,
    kind: FlowSegmentKind,
}

#[derive(Debug, Clone)]
struct LocationHeatPoint {
    anchor: Vec3,
    intensity: i64,
}

pub(super) fn spawn_world_overlay_controls(
    parent: &mut ChildSpawnerCommands,
    font: Handle<Font>,
    locale: UiLocale,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.14, 0.17)),
            BorderColor::all(Color::srgb(0.22, 0.26, 0.31)),
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(24.0),
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|buttons| {
                spawn_overlay_button(
                    buttons,
                    &font,
                    WorldOverlayKind::Chunk,
                    overlay_button_label("chunk", locale),
                    Color::srgb(0.25, 0.31, 0.37),
                );
                spawn_overlay_button(
                    buttons,
                    &font,
                    WorldOverlayKind::Heat,
                    overlay_button_label("heat", locale),
                    Color::srgb(0.35, 0.28, 0.14),
                );
                spawn_overlay_button(
                    buttons,
                    &font,
                    WorldOverlayKind::Flow,
                    overlay_button_label("flow", locale),
                    Color::srgb(0.2, 0.26, 0.38),
                );
            });

            root.spawn((
                Text::new(overlay_loading(locale)),
                TextFont {
                    font,
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.76, 0.8, 0.9)),
                WorldOverlayStatusText,
            ));
        });
}

fn spawn_overlay_button(
    buttons: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    kind: WorldOverlayKind,
    label: &str,
    color: Color,
) {
    buttons
        .spawn((
            Button,
            Node {
                padding: UiRect::horizontal(Val::Px(8.0)),
                height: Val::Px(22.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color),
            WorldOverlayToggleButton { kind },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                WorldOverlayToggleLabel { kind },
            ));
        });
}

pub(super) fn update_world_overlay_toggle_labels(
    i18n: Option<Res<UiI18n>>,
    mut query: Query<(&WorldOverlayToggleLabel, &mut Text)>,
) {
    let Some(i18n) = i18n else {
        return;
    };
    if !i18n.is_changed() {
        return;
    }

    let locale = i18n.locale;
    for (label, mut text) in &mut query {
        text.0 = match label.kind {
            WorldOverlayKind::Chunk => overlay_button_label("chunk", locale),
            WorldOverlayKind::Heat => overlay_button_label("heat", locale),
            WorldOverlayKind::Flow => overlay_button_label("flow", locale),
        }
        .to_string();
    }
}

pub(super) fn handle_world_overlay_toggle_buttons(
    mut config: ResMut<WorldOverlayConfig>,
    mut interactions: Query<
        (&Interaction, &WorldOverlayToggleButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button.kind {
            WorldOverlayKind::Chunk => config.show_chunk_overlay = !config.show_chunk_overlay,
            WorldOverlayKind::Heat => {
                config.show_resource_heatmap = !config.show_resource_heatmap;
            }
            WorldOverlayKind::Flow => config.show_flow_overlay = !config.show_flow_overlay,
        }
    }
}

pub(super) fn update_world_overlay_status_text(
    state: Res<ViewerState>,
    config: Res<WorldOverlayConfig>,
    i18n: Option<Res<UiI18n>>,
    mut ui_state: ResMut<WorldOverlayUiState>,
    mut text_query: Query<&mut Text, With<WorldOverlayStatusText>>,
) {
    let locale_changed = i18n
        .as_ref()
        .map(|value| value.is_changed())
        .unwrap_or(false);
    if !state.is_changed() && !config.is_changed() && !locale_changed {
        return;
    }

    let locale = locale_or_default(i18n.as_deref());

    let summary =
        build_overlay_status_text(state.snapshot.as_ref(), &state.events, *config, locale);
    ui_state.status_text = summary.clone();

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = summary;
    }
}

pub(super) fn update_world_overlays_3d(
    mut commands: Commands,
    state: Res<ViewerState>,
    overlay_config: Res<WorldOverlayConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
    mut chunk_visibility: Query<&mut Visibility>,
) {
    if !state.is_changed() && !overlay_config.is_changed() {
        return;
    }

    let chunk_visibility_value = if overlay_config.show_chunk_overlay {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for entity in scene.chunk_entities.values() {
        if let Ok(mut visibility) = chunk_visibility.get_mut(*entity) {
            *visibility = chunk_visibility_value;
        }
    }

    for entity in scene.heat_overlay_entities.drain(..) {
        if let Ok(mut command) = commands.get_entity(entity) {
            command.despawn();
        }
    }
    for entity in scene.flow_overlay_entities.drain(..) {
        if let Ok(mut command) = commands.get_entity(entity) {
            command.despawn();
        }
    }

    let Some(snapshot) = state.snapshot.as_ref() else {
        return;
    };
    let Some(origin) = scene.origin else {
        return;
    };

    if overlay_config.show_resource_heatmap {
        let heat_points = collect_location_heat_points(snapshot, origin, DEFAULT_CM_TO_UNIT);
        let max_intensity = heat_points
            .iter()
            .map(|point| point.intensity.max(0))
            .max()
            .unwrap_or(1)
            .max(1);

        for point in heat_points {
            let ratio = (point.intensity.max(0) as f32 / max_intensity as f32).clamp(0.0, 1.0);
            let height = HEAT_BASE_HEIGHT + ratio * HEAT_MAX_HEIGHT;
            let material = if ratio >= 0.75 {
                assets.heat_high_material.clone()
            } else if ratio >= 0.35 {
                assets.heat_mid_material.clone()
            } else {
                assets.heat_low_material.clone()
            };
            let entity = commands
                .spawn((
                    Mesh3d(assets.world_box_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(
                        point.anchor + Vec3::Y * (HEAT_OFFSET_Y + height * 0.5),
                    )
                    .with_scale(Vec3::new(0.22, height, 0.22)),
                    Name::new("overlay:heat"),
                ))
                .id();
            scene.heat_overlay_entities.push(entity);
        }
    }

    if overlay_config.show_flow_overlay {
        let mut flow_segments =
            collect_flow_segments(snapshot, &state.events, origin, DEFAULT_CM_TO_UNIT);
        let max_amount = flow_segments
            .iter()
            .map(|segment| segment.amount.abs())
            .max()
            .unwrap_or(1)
            .max(1);

        for segment in flow_segments.drain(..) {
            let ratio = (segment.amount.abs() as f32 / max_amount as f32).clamp(0.0, 1.0);
            let thickness = FLOW_MIN_THICKNESS + ratio * (FLOW_MAX_THICKNESS - FLOW_MIN_THICKNESS);
            let material = match segment.kind {
                FlowSegmentKind::Power => assets.flow_power_material.clone(),
                FlowSegmentKind::Trade => assets.flow_trade_material.clone(),
            };
            let entity = commands
                .spawn((
                    Mesh3d(assets.world_box_mesh.clone()),
                    MeshMaterial3d(material),
                    line_transform(segment.from, segment.to, thickness),
                    Name::new("overlay:flow"),
                ))
                .id();
            scene.flow_overlay_entities.push(entity);
        }
    }
}

fn build_overlay_status_text(
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
    config: WorldOverlayConfig,
    locale: UiLocale,
) -> String {
    let Some(snapshot) = snapshot else {
        return overlay_status(
            None,
            None,
            0,
            config.show_chunk_overlay,
            config.show_resource_heatmap,
            config.show_flow_overlay,
            locale,
        );
    };

    let (unexplored, generated, exhausted) = chunk_state_counts(snapshot);
    let heat_peak = top_heat_location(snapshot)
        .map(|(id, value)| format!("{id}:{value}"))
        .unwrap_or_else(|| "-".to_string());
    let flow_count = collect_flow_segments(
        snapshot,
        events,
        space_origin(&snapshot.config.space),
        DEFAULT_CM_TO_UNIT,
    )
    .len();

    overlay_status(
        Some((unexplored, generated, exhausted)),
        Some(heat_peak),
        flow_count,
        config.show_chunk_overlay,
        config.show_resource_heatmap,
        config.show_flow_overlay,
        locale,
    )
}

fn chunk_state_counts(snapshot: &WorldSnapshot) -> (usize, usize, usize) {
    let mut unexplored = 0;
    let mut generated = 0;
    let mut exhausted = 0;
    for state in snapshot.model.chunks.values() {
        match state {
            ChunkState::Unexplored => unexplored += 1,
            ChunkState::Generated => generated += 1,
            ChunkState::Exhausted => exhausted += 1,
        }
    }
    (unexplored, generated, exhausted)
}

fn top_heat_location(snapshot: &WorldSnapshot) -> Option<(String, i64)> {
    snapshot
        .model
        .locations
        .iter()
        .map(|(location_id, location)| {
            let electricity = location.resources.get(ResourceKind::Electricity).max(0);
            let hardware = location.resources.get(ResourceKind::Hardware).max(0);
            let data = location.resources.get(ResourceKind::Data).max(0);
            let score = electricity
                .saturating_add(hardware.saturating_mul(4))
                .saturating_add(data.saturating_mul(2));
            (location_id.clone(), score)
        })
        .max_by_key(|(_, score)| *score)
}

fn collect_location_heat_points(
    snapshot: &WorldSnapshot,
    origin: GeoPos,
    cm_to_unit: f32,
) -> Vec<LocationHeatPoint> {
    snapshot
        .model
        .locations
        .values()
        .map(|location| {
            let electricity = location.resources.get(ResourceKind::Electricity).max(0);
            let hardware = location.resources.get(ResourceKind::Hardware).max(0);
            let data = location.resources.get(ResourceKind::Data).max(0);
            let intensity = electricity
                .saturating_add(hardware.saturating_mul(4))
                .saturating_add(data.saturating_mul(2));
            LocationHeatPoint {
                anchor: geo_to_vec3(location.pos, origin, cm_to_unit),
                intensity,
            }
        })
        .collect()
}

fn collect_flow_segments(
    snapshot: &WorldSnapshot,
    events: &[WorldEvent],
    origin: GeoPos,
    cm_to_unit: f32,
) -> Vec<FlowSegment> {
    let mut segments = Vec::new();

    for event in events.iter().rev().take(FLOW_WINDOW) {
        match &event.kind {
            WorldEventKind::ResourceTransferred {
                from, to, amount, ..
            } => {
                let from_pos = owner_position(snapshot, from, origin, cm_to_unit);
                let to_pos = owner_position(snapshot, to, origin, cm_to_unit);
                if let (Some(from_pos), Some(to_pos)) = (from_pos, to_pos) {
                    if from_pos.distance(to_pos) > 0.00001 {
                        segments.push(FlowSegment {
                            from: from_pos + Vec3::Y * FLOW_OFFSET_Y,
                            to: to_pos + Vec3::Y * FLOW_OFFSET_Y,
                            amount: amount.abs(),
                            kind: FlowSegmentKind::Trade,
                        });
                    }
                }
            }
            WorldEventKind::Power(PowerEvent::PowerTransferred {
                from,
                to,
                amount,
                loss,
                ..
            }) => {
                let from_pos = owner_position(snapshot, from, origin, cm_to_unit);
                let to_pos = owner_position(snapshot, to, origin, cm_to_unit);
                if let (Some(from_pos), Some(to_pos)) = (from_pos, to_pos) {
                    if from_pos.distance(to_pos) > 0.00001 {
                        segments.push(FlowSegment {
                            from: from_pos + Vec3::Y * FLOW_OFFSET_Y,
                            to: to_pos + Vec3::Y * FLOW_OFFSET_Y,
                            amount: amount.abs().saturating_add(loss.abs()),
                            kind: FlowSegmentKind::Power,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    segments
}

fn owner_position(
    snapshot: &WorldSnapshot,
    owner: &ResourceOwner,
    origin: GeoPos,
    cm_to_unit: f32,
) -> Option<Vec3> {
    match owner {
        ResourceOwner::Agent { agent_id } => snapshot
            .model
            .agents
            .get(agent_id)
            .map(|agent| geo_to_vec3(agent.pos, origin, cm_to_unit)),
        ResourceOwner::Location { location_id } => snapshot
            .model
            .locations
            .get(location_id)
            .map(|location| geo_to_vec3(location.pos, origin, cm_to_unit)),
    }
}

fn line_transform(from: Vec3, to: Vec3, thickness: f32) -> Transform {
    let delta = to - from;
    let length = delta.length().max(0.0001);
    let direction = delta / length;
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);
    Transform {
        translation: (from + to) * 0.5,
        rotation,
        scale: Vec3::new(thickness, length, thickness),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        Agent, ChunkRuntimeConfig, Location, PowerEvent, WorldConfig, WorldModel,
    };

    fn sample_snapshot() -> WorldSnapshot {
        let mut model = WorldModel::default();

        let mut loc_a = Location::new("loc-a", "A", GeoPos::new(0.0, 0.0, 0.0));
        loc_a
            .resources
            .set(ResourceKind::Electricity, 20)
            .expect("set electricity");
        let mut loc_b = Location::new("loc-b", "B", GeoPos::new(100.0, 0.0, 0.0));
        loc_b
            .resources
            .set(ResourceKind::Electricity, 80)
            .expect("set electricity");

        model.locations.insert("loc-a".to_string(), loc_a);
        model.locations.insert("loc-b".to_string(), loc_b);
        model.agents.insert(
            "agent-1".to_string(),
            Agent::new("agent-1", "loc-a", GeoPos::new(0.0, 0.0, 0.0)),
        );

        WorldSnapshot {
            version: agent_world::simulator::SNAPSHOT_VERSION,
            chunk_generation_schema_version:
                agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
            time: 5,
            config: WorldConfig::default(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: 2,
            next_action_id: 2,
            pending_actions: Vec::new(),
            journal_len: 0,
        }
    }

    #[test]
    fn overlay_status_contains_chunk_heat_and_flow() {
        let mut snapshot = sample_snapshot();
        snapshot.model.chunks.insert(
            agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
            ChunkState::Generated,
        );

        let events = vec![WorldEvent {
            id: 1,
            time: 4,
            kind: WorldEventKind::Power(PowerEvent::PowerTransferred {
                from: ResourceOwner::Location {
                    location_id: "loc-a".to_string(),
                },
                to: ResourceOwner::Location {
                    location_id: "loc-b".to_string(),
                },
                amount: 10,
                loss: 1,
                price_per_pu: 0,
            }),
        }];

        let text = build_overlay_status_text(
            Some(&snapshot),
            &events,
            WorldOverlayConfig::default(),
            UiLocale::EnUs,
        );
        assert!(text.contains("Overlay[chunk:on heat:on flow:on]"));
        assert!(text.contains("chunks(u/g/e)=0/1/0"));
        assert!(text.contains("heat_peak=loc-b:80"));
        assert!(text.contains("flows=1"));
    }

    #[test]
    fn collect_flow_segments_extracts_trade_and_power() {
        let snapshot = sample_snapshot();
        let origin = space_origin(&snapshot.config.space);
        let events = vec![
            WorldEvent {
                id: 1,
                time: 1,
                kind: WorldEventKind::ResourceTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    to: ResourceOwner::Location {
                        location_id: "loc-b".to_string(),
                    },
                    kind: ResourceKind::Hardware,
                    amount: 5,
                },
            },
            WorldEvent {
                id: 2,
                time: 2,
                kind: WorldEventKind::Power(PowerEvent::PowerTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-b".to_string(),
                    },
                    to: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    amount: 9,
                    loss: 2,
                    price_per_pu: 0,
                }),
            },
        ];

        let segments = collect_flow_segments(&snapshot, &events, origin, DEFAULT_CM_TO_UNIT);
        assert_eq!(segments.len(), 2);
        assert!(segments
            .iter()
            .any(|segment| segment.kind == FlowSegmentKind::Trade));
        assert!(segments
            .iter()
            .any(|segment| segment.kind == FlowSegmentKind::Power));
    }

    #[test]
    fn overlay_toggle_button_flips_flags() {
        let mut app = App::new();
        app.add_systems(Update, handle_world_overlay_toggle_buttons);
        app.world_mut()
            .insert_resource(WorldOverlayConfig::default());

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            WorldOverlayToggleButton {
                kind: WorldOverlayKind::Heat,
            },
        ));

        app.update();

        let config = app.world().resource::<WorldOverlayConfig>();
        assert!(!config.show_resource_heatmap);
        assert!(config.show_chunk_overlay);
        assert!(config.show_flow_overlay);
    }
}
