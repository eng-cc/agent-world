use agent_world::simulator::{
    ChunkCoord, PowerEvent, RejectReason, ResourceOwner, WorldEvent, WorldEventKind,
};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::*;

#[derive(Resource)]
pub(super) struct EventObjectLinkState {
    pub message: String,
}

impl Default for EventObjectLinkState {
    fn default() -> Self {
        Self {
            message: "Link: ready".to_string(),
        }
    }
}

#[derive(Component)]
pub(super) struct LocateFocusEventButton;

#[derive(Component)]
pub(super) struct JumpSelectionEventsButton;

#[derive(Component)]
pub(super) struct EventObjectLinkText;

#[derive(Clone)]
pub(super) struct SelectionTarget {
    pub(super) kind: SelectionKind,
    pub(super) id: String,
    pub(super) name: Option<String>,
}

pub(super) fn spawn_event_object_link_controls(
    parent: &mut ChildSpawnerCommands,
    font: Handle<Font>,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            row_gap: Val::Px(6.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                column_gap: Val::Px(6.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|buttons| {
                spawn_link_button(
                    buttons,
                    &font,
                    "Locate Focus Event Object",
                    LocateFocusEventButton,
                    Color::srgb(0.22, 0.32, 0.24),
                );
                spawn_link_button(
                    buttons,
                    &font,
                    "Jump Selection Events",
                    JumpSelectionEventsButton,
                    Color::srgb(0.22, 0.24, 0.34),
                );
            });

            root.spawn((
                Text::new("Link: ready"),
                TextFont {
                    font,
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.9)),
                EventObjectLinkText,
            ));
        });
}

fn spawn_link_button<C: Component>(
    buttons: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    marker: C,
    background: Color,
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
            BackgroundColor(background),
            marker,
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
            ));
        });
}

pub(super) fn handle_locate_focus_event_button(
    mut interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<LocateFocusEventButton>,
        ),
    >,
    state: Res<ViewerState>,
    scene: Res<Viewer3dScene>,
    config: Res<Viewer3dConfig>,
    mut selection: ResMut<ViewerSelection>,
    mut link_state: ResMut<EventObjectLinkState>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>)>,
    mut timeline: Option<ResMut<TimelineUiState>>,
) {
    for interaction in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let focus_tick = focus_tick(&state, timeline.as_deref());
        let Some(event) = nearest_event_to_tick(&state.events, focus_tick) else {
            link_state.message = "Link: no events available".to_string();
            continue;
        };

        let Some(target) = event_primary_target(event, state.snapshot.as_ref()) else {
            link_state.message = format!(
                "Link: event #{} t{} has no mappable object",
                event.id, event.time
            );
            continue;
        };

        let Some(entity) = target_entity(&scene, &target) else {
            link_state.message = format!(
                "Link: target {} {} is not in current scene",
                selection_kind_label(target.kind),
                target.id
            );
            continue;
        };

        apply_selection(
            &mut selection,
            &mut transforms,
            &config,
            entity,
            target.kind,
            target.id.clone(),
            target.name.clone(),
        );

        if let Some(timeline) = timeline.as_mut() {
            timeline.target_tick = event.time;
            timeline.manual_override = true;
            timeline.drag_active = false;
        }

        link_state.message = format!(
            "Link: event #{} t{} -> {} {}",
            event.id,
            event.time,
            selection_kind_label(target.kind),
            target.id
        );
    }
}

pub(super) fn handle_jump_selection_events_button(
    mut interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<JumpSelectionEventsButton>,
        ),
    >,
    state: Res<ViewerState>,
    selection: Res<ViewerSelection>,
    mut link_state: ResMut<EventObjectLinkState>,
    mut timeline: Option<ResMut<TimelineUiState>>,
) {
    for interaction in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(current) = selection.current.as_ref() else {
            link_state.message = "Link: no selection".to_string();
            continue;
        };

        let related_ticks =
            selection_related_ticks(current, &state.events, state.snapshot.as_ref());
        if related_ticks.is_empty() {
            link_state.message = format!(
                "Link: {} {} has no related events",
                selection_kind_label(current.kind),
                current.id
            );
            continue;
        }

        let pivot = focus_tick(&state, timeline.as_deref());
        let Some(next_tick) = select_next_tick(&related_ticks, pivot) else {
            link_state.message = "Link: no target tick".to_string();
            continue;
        };

        if let Some(timeline) = timeline.as_mut() {
            timeline.target_tick = next_tick;
            timeline.manual_override = true;
            timeline.drag_active = false;
        }

        link_state.message = format!(
            "Link: {} {} -> t{}",
            selection_kind_label(current.kind),
            current.id,
            next_tick
        );
    }
}

pub(super) fn update_event_object_link_text(
    link_state: Res<EventObjectLinkState>,
    mut query: Query<&mut Text, With<EventObjectLinkText>>,
) {
    if !link_state.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        text.0 = link_state.message.clone();
    }
}

pub(super) fn pick_3d_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Viewer3dCamera>>,
    agents: Query<(Entity, &GlobalTransform, &AgentMarker)>,
    locations: Query<(Entity, &GlobalTransform, &LocationMarker)>,
    assets: Query<(Entity, &GlobalTransform, &AssetMarker)>,
    power_plants: Query<(Entity, &GlobalTransform, &PowerPlantMarker)>,
    power_storages: Query<(Entity, &GlobalTransform, &PowerStorageMarker)>,
    chunks: Query<(Entity, &GlobalTransform, &ChunkMarker)>,
    config: Res<Viewer3dConfig>,
    mut selection: ResMut<ViewerSelection>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    if cursor_position.x > (window.width() - UI_PANEL_WIDTH) {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let mut best: Option<(Entity, SelectionKind, String, Option<String>, f32)> = None;

    for (entity, transform, marker) in agents.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::Agent,
                    marker.id.clone(),
                    None,
                    distance,
                ));
            }
        }
    }

    for (entity, transform, marker) in locations.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::Location,
                    marker.id.clone(),
                    Some(marker.name.clone()),
                    distance,
                ));
            }
        }
    }

    for (entity, transform, marker) in assets.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::Asset,
                    marker.id.clone(),
                    None,
                    distance,
                ));
            }
        }
    }

    for (entity, transform, marker) in power_plants.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::PowerPlant,
                    marker.id.clone(),
                    None,
                    distance,
                ));
            }
        }
    }

    for (entity, transform, marker) in power_storages.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::PowerStorage,
                    marker.id.clone(),
                    None,
                    distance,
                ));
            }
        }
    }

    for (entity, transform, marker) in chunks.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= CHUNK_PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::Chunk,
                    marker.id.clone(),
                    Some(marker.state.clone()),
                    distance,
                ));
            }
        }
    }

    if let Some((entity, kind, id, name, _)) = best {
        apply_selection(
            &mut selection,
            &mut transforms,
            &config,
            entity,
            kind,
            id,
            name,
        );
    } else if selection.current.is_some() {
        if let Some(current) = selection.current.take() {
            reset_entity_scale(&mut transforms, current.entity);
        }
    }
}

fn apply_selection(
    selection: &mut ViewerSelection,
    transforms: &mut Query<(&mut Transform, Option<&BaseScale>)>,
    config: &Viewer3dConfig,
    entity: Entity,
    kind: SelectionKind,
    id: String,
    name: Option<String>,
) {
    if let Some(current) = selection.current.take() {
        reset_entity_scale(transforms, current.entity);
    }
    selection.current = Some(SelectionInfo {
        entity,
        kind,
        id,
        name,
    });
    if config.highlight_selected {
        apply_entity_highlight(transforms, entity);
    }
}

fn focus_tick(state: &ViewerState, timeline: Option<&TimelineUiState>) -> u64 {
    match timeline {
        Some(timeline) if timeline.manual_override || timeline.drag_active => timeline.target_tick,
        _ => current_tick_from_state(state),
    }
}

fn current_tick_from_state(state: &ViewerState) -> u64 {
    state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0)
}

fn nearest_event_to_tick(events: &[WorldEvent], tick: u64) -> Option<&WorldEvent> {
    events.iter().min_by_key(|event| event.time.abs_diff(tick))
}

fn select_next_tick(ticks: &[u64], pivot: u64) -> Option<u64> {
    ticks
        .iter()
        .copied()
        .find(|tick| *tick > pivot)
        .or_else(|| ticks.first().copied())
}

fn selection_related_ticks(
    selection: &SelectionInfo,
    events: &[WorldEvent],
    snapshot: Option<&WorldSnapshot>,
) -> Vec<u64> {
    let mut ticks = Vec::new();
    for event in events {
        if event_matches_selection(event, selection, snapshot) {
            ticks.push(event.time);
        }
    }
    ticks.sort_unstable();
    ticks.dedup();
    ticks
}

fn event_matches_selection(
    event: &WorldEvent,
    selection: &SelectionInfo,
    snapshot: Option<&WorldSnapshot>,
) -> bool {
    match selection.kind {
        SelectionKind::Agent => event_matches_agent(event, selection.id.as_str()),
        SelectionKind::Location => event_matches_location(event, selection.id.as_str()),
        SelectionKind::PowerPlant => event_matches_power_plant(event, selection.id.as_str()),
        SelectionKind::PowerStorage => event_matches_power_storage(event, selection.id.as_str()),
        SelectionKind::Chunk => selection
            .id
            .parse::<ChunkCoordId>()
            .ok()
            .map(|coord| event_matches_chunk(event, coord.coord))
            .unwrap_or(false),
        SelectionKind::Asset => snapshot
            .and_then(|snapshot| snapshot.model.assets.get(selection.id.as_str()))
            .map(|asset| event_matches_owner(event, &asset.owner))
            .unwrap_or(false),
    }
}

pub(super) fn event_primary_target(
    event: &WorldEvent,
    snapshot: Option<&WorldSnapshot>,
) -> Option<SelectionTarget> {
    match &event.kind {
        WorldEventKind::LocationRegistered {
            location_id, name, ..
        } => Some(SelectionTarget {
            kind: SelectionKind::Location,
            id: location_id.clone(),
            name: Some(name.clone()),
        }),
        WorldEventKind::AgentRegistered { agent_id, .. }
        | WorldEventKind::AgentMoved { agent_id, .. }
        | WorldEventKind::RadiationHarvested { agent_id, .. } => Some(SelectionTarget {
            kind: SelectionKind::Agent,
            id: agent_id.clone(),
            name: None,
        }),
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            owner_to_target(from, snapshot).or_else(|| owner_to_target(to, snapshot))
        }
        WorldEventKind::CompoundRefined { owner, .. } => owner_to_target(owner, snapshot),
        WorldEventKind::ChunkGenerated { coord, .. } => Some(SelectionTarget {
            kind: SelectionKind::Chunk,
            id: chunk_id(*coord),
            name: None,
        }),
        WorldEventKind::ModuleVisualEntityUpserted { entity } => Some(SelectionTarget {
            kind: SelectionKind::Asset,
            id: entity.entity_id.clone(),
            name: None,
        }),
        WorldEventKind::ModuleVisualEntityRemoved { entity_id } => Some(SelectionTarget {
            kind: SelectionKind::Asset,
            id: entity_id.clone(),
            name: None,
        }),
        WorldEventKind::ActionRejected { reason } => reject_reason_to_target(reason, snapshot),
        WorldEventKind::Power(power_event) => power_event_target(power_event, snapshot),
    }
}

fn power_event_target(
    power_event: &PowerEvent,
    snapshot: Option<&WorldSnapshot>,
) -> Option<SelectionTarget> {
    match power_event {
        PowerEvent::PowerPlantRegistered { plant } => Some(SelectionTarget {
            kind: SelectionKind::PowerPlant,
            id: plant.id.clone(),
            name: None,
        }),
        PowerEvent::PowerStorageRegistered { storage } => Some(SelectionTarget {
            kind: SelectionKind::PowerStorage,
            id: storage.id.clone(),
            name: None,
        }),
        PowerEvent::PowerGenerated { plant_id, .. } => Some(SelectionTarget {
            kind: SelectionKind::PowerPlant,
            id: plant_id.clone(),
            name: None,
        }),
        PowerEvent::PowerStored { storage_id, .. }
        | PowerEvent::PowerDischarged { storage_id, .. } => Some(SelectionTarget {
            kind: SelectionKind::PowerStorage,
            id: storage_id.clone(),
            name: None,
        }),
        PowerEvent::PowerConsumed { agent_id, .. }
        | PowerEvent::PowerStateChanged { agent_id, .. }
        | PowerEvent::PowerCharged { agent_id, .. } => Some(SelectionTarget {
            kind: SelectionKind::Agent,
            id: agent_id.clone(),
            name: None,
        }),
        PowerEvent::PowerTransferred { from, to, .. } => {
            owner_to_target(from, snapshot).or_else(|| owner_to_target(to, snapshot))
        }
    }
}

fn reject_reason_to_target(
    reason: &RejectReason,
    snapshot: Option<&WorldSnapshot>,
) -> Option<SelectionTarget> {
    match reason {
        RejectReason::AgentAlreadyExists { agent_id }
        | RejectReason::AgentNotFound { agent_id }
        | RejectReason::AgentAlreadyAtLocation { agent_id, .. }
        | RejectReason::AgentNotAtLocation { agent_id, .. }
        | RejectReason::AgentShutdown { agent_id } => Some(SelectionTarget {
            kind: SelectionKind::Agent,
            id: agent_id.clone(),
            name: None,
        }),
        RejectReason::AgentsNotCoLocated { agent_id, .. } => Some(SelectionTarget {
            kind: SelectionKind::Agent,
            id: agent_id.clone(),
            name: None,
        }),
        RejectReason::LocationAlreadyExists { location_id }
        | RejectReason::LocationNotFound { location_id }
        | RejectReason::RadiationUnavailable { location_id } => {
            location_target(location_id.as_str(), snapshot)
        }
        RejectReason::FacilityAlreadyExists { facility_id }
        | RejectReason::FacilityNotFound { facility_id } => facility_target(facility_id, snapshot),
        RejectReason::InsufficientResource { owner, .. } => owner_to_target(owner, snapshot),
        RejectReason::LocationTransferNotAllowed { from, .. } => {
            location_target(from.as_str(), snapshot)
        }
        RejectReason::ChunkGenerationFailed { x, y, z } => Some(SelectionTarget {
            kind: SelectionKind::Chunk,
            id: format!("{x},{y},{z}"),
            name: None,
        }),
        _ => None,
    }
}

fn owner_to_target(
    owner: &ResourceOwner,
    snapshot: Option<&WorldSnapshot>,
) -> Option<SelectionTarget> {
    match owner {
        ResourceOwner::Agent { agent_id } => Some(SelectionTarget {
            kind: SelectionKind::Agent,
            id: agent_id.clone(),
            name: None,
        }),
        ResourceOwner::Location { location_id } => location_target(location_id, snapshot),
    }
}

fn location_target(location_id: &str, snapshot: Option<&WorldSnapshot>) -> Option<SelectionTarget> {
    let name = snapshot
        .and_then(|snapshot| snapshot.model.locations.get(location_id))
        .map(|location| location.name.clone());
    Some(SelectionTarget {
        kind: SelectionKind::Location,
        id: location_id.to_string(),
        name,
    })
}

fn facility_target(facility_id: &str, snapshot: Option<&WorldSnapshot>) -> Option<SelectionTarget> {
    if let Some(snapshot) = snapshot {
        if snapshot.model.power_plants.contains_key(facility_id) {
            return Some(SelectionTarget {
                kind: SelectionKind::PowerPlant,
                id: facility_id.to_string(),
                name: None,
            });
        }
        if snapshot.model.power_storages.contains_key(facility_id) {
            return Some(SelectionTarget {
                kind: SelectionKind::PowerStorage,
                id: facility_id.to_string(),
                name: None,
            });
        }
    }
    None
}

pub(super) fn target_entity(scene: &Viewer3dScene, target: &SelectionTarget) -> Option<Entity> {
    match target.kind {
        SelectionKind::Agent => scene.agent_entities.get(target.id.as_str()).copied(),
        SelectionKind::Location => scene.location_entities.get(target.id.as_str()).copied(),
        SelectionKind::Asset => scene
            .asset_entities
            .get(target.id.as_str())
            .copied()
            .or_else(|| {
                scene
                    .module_visual_entities
                    .get(target.id.as_str())
                    .copied()
            }),
        SelectionKind::PowerPlant => scene.power_plant_entities.get(target.id.as_str()).copied(),
        SelectionKind::PowerStorage => scene
            .power_storage_entities
            .get(target.id.as_str())
            .copied(),
        SelectionKind::Chunk => scene.chunk_entities.get(target.id.as_str()).copied(),
    }
}

fn event_matches_agent(event: &WorldEvent, agent_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::AgentRegistered { agent_id: id, .. }
        | WorldEventKind::AgentMoved { agent_id: id, .. }
        | WorldEventKind::RadiationHarvested { agent_id: id, .. } => id == agent_id,
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            owner_is_agent(from, agent_id) || owner_is_agent(to, agent_id)
        }
        WorldEventKind::CompoundRefined { owner, .. } => owner_is_agent(owner, agent_id),
        WorldEventKind::ActionRejected { reason } => reject_reason_matches_agent(reason, agent_id),
        WorldEventKind::Power(power_event) => match power_event {
            PowerEvent::PowerConsumed { agent_id: id, .. }
            | PowerEvent::PowerStateChanged { agent_id: id, .. }
            | PowerEvent::PowerCharged { agent_id: id, .. } => id == agent_id,
            _ => false,
        },
        _ => false,
    }
}

fn event_matches_location(event: &WorldEvent, location_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::LocationRegistered {
            location_id: id, ..
        } => id == location_id,
        WorldEventKind::AgentRegistered {
            location_id: id, ..
        }
        | WorldEventKind::RadiationHarvested {
            location_id: id, ..
        } => id == location_id,
        WorldEventKind::AgentMoved { from, to, .. } => from == location_id || to == location_id,
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            owner_is_location(from, location_id) || owner_is_location(to, location_id)
        }
        WorldEventKind::CompoundRefined { owner, .. } => owner_is_location(owner, location_id),
        WorldEventKind::ActionRejected { reason } => {
            reject_reason_matches_location(reason, location_id)
        }
        WorldEventKind::Power(power_event) => match power_event {
            PowerEvent::PowerGenerated {
                location_id: id, ..
            }
            | PowerEvent::PowerStored {
                location_id: id, ..
            }
            | PowerEvent::PowerDischarged {
                location_id: id, ..
            } => id == location_id,
            PowerEvent::PowerPlantRegistered { plant } => plant.location_id == location_id,
            PowerEvent::PowerStorageRegistered { storage } => storage.location_id == location_id,
            _ => false,
        },
        _ => false,
    }
}

fn event_matches_power_plant(event: &WorldEvent, facility_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerPlantRegistered { plant }) => {
            plant.id == facility_id
        }
        WorldEventKind::Power(PowerEvent::PowerGenerated { plant_id, .. }) => {
            plant_id == facility_id
        }
        WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityAlreadyExists { facility_id: id },
        }
        | WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityNotFound { facility_id: id },
        } => id == facility_id,
        _ => false,
    }
}

fn event_matches_power_storage(event: &WorldEvent, facility_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerStorageRegistered { storage }) => {
            storage.id == facility_id
        }
        WorldEventKind::Power(PowerEvent::PowerStored { storage_id, .. })
        | WorldEventKind::Power(PowerEvent::PowerDischarged { storage_id, .. }) => {
            storage_id == facility_id
        }
        WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityAlreadyExists { facility_id: id },
        }
        | WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityNotFound { facility_id: id },
        } => id == facility_id,
        _ => false,
    }
}

fn event_matches_chunk(event: &WorldEvent, coord: ChunkCoord) -> bool {
    match &event.kind {
        WorldEventKind::ChunkGenerated {
            coord: event_coord, ..
        } => *event_coord == coord,
        WorldEventKind::ActionRejected {
            reason: RejectReason::ChunkGenerationFailed { x, y, z },
        } => *x == coord.x && *y == coord.y && *z == coord.z,
        _ => false,
    }
}

fn event_matches_owner(event: &WorldEvent, owner: &ResourceOwner) -> bool {
    match &event.kind {
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            from == owner || to == owner
        }
        WorldEventKind::CompoundRefined {
            owner: event_owner, ..
        } => event_owner == owner,
        WorldEventKind::ActionRejected {
            reason:
                RejectReason::InsufficientResource {
                    owner: event_owner, ..
                },
        } => event_owner == owner,
        _ => false,
    }
}

fn reject_reason_matches_agent(reason: &RejectReason, agent_id: &str) -> bool {
    match reason {
        RejectReason::AgentAlreadyExists { agent_id: id }
        | RejectReason::AgentNotFound { agent_id: id }
        | RejectReason::AgentAlreadyAtLocation { agent_id: id, .. }
        | RejectReason::AgentNotAtLocation { agent_id: id, .. }
        | RejectReason::AgentShutdown { agent_id: id } => id == agent_id,
        RejectReason::AgentsNotCoLocated {
            agent_id: id,
            other_agent_id,
        } => id == agent_id || other_agent_id == agent_id,
        RejectReason::InsufficientResource { owner, .. } => owner_is_agent(owner, agent_id),
        _ => false,
    }
}

fn reject_reason_matches_location(reason: &RejectReason, location_id: &str) -> bool {
    match reason {
        RejectReason::LocationAlreadyExists { location_id: id }
        | RejectReason::LocationNotFound { location_id: id }
        | RejectReason::RadiationUnavailable { location_id: id } => id == location_id,
        RejectReason::LocationTransferNotAllowed { from, to } => {
            from == location_id || to == location_id
        }
        RejectReason::AgentAlreadyAtLocation {
            location_id: id, ..
        }
        | RejectReason::AgentNotAtLocation {
            location_id: id, ..
        } => id == location_id,
        RejectReason::InsufficientResource { owner, .. } => owner_is_location(owner, location_id),
        _ => false,
    }
}

fn owner_is_agent(owner: &ResourceOwner, agent_id: &str) -> bool {
    matches!(owner, ResourceOwner::Agent { agent_id: id } if id == agent_id)
}

fn owner_is_location(owner: &ResourceOwner, location_id: &str) -> bool {
    matches!(owner, ResourceOwner::Location { location_id: id } if id == location_id)
}

fn selection_kind_label(kind: SelectionKind) -> &'static str {
    match kind {
        SelectionKind::Agent => "agent",
        SelectionKind::Location => "location",
        SelectionKind::Asset => "asset",
        SelectionKind::PowerPlant => "power_plant",
        SelectionKind::PowerStorage => "power_storage",
        SelectionKind::Chunk => "chunk",
    }
}

fn chunk_id(coord: ChunkCoord) -> String {
    format!("{},{},{}", coord.x, coord.y, coord.z)
}

#[derive(Clone, Copy)]
struct ChunkCoordId {
    coord: ChunkCoord,
}

impl std::str::FromStr for ChunkCoordId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split(',');
        let x = parts.next().and_then(|part| part.parse::<i32>().ok());
        let y = parts.next().and_then(|part| part.parse::<i32>().ok());
        let z = parts.next().and_then(|part| part.parse::<i32>().ok());
        if x.is_none() || y.is_none() || z.is_none() || parts.next().is_some() {
            return Err(());
        }
        Ok(Self {
            coord: ChunkCoord {
                x: x.ok_or(())?,
                y: y.ok_or(())?,
                z: z.ok_or(())?,
            },
        })
    }
}

#[cfg(test)]
mod tests;
