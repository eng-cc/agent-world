use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::simulator::{
    chunk_bounds, AgentDecisionTrace, ChunkCoord, PowerEvent, ResourceOwner, WorldEvent,
    WorldEventKind, WorldSnapshot,
};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use serde::Serialize;
use serde_json::{json, Value};

use super::ui_text::selection_details_summary;
use super::*;

const RELATED_EVENT_LIMIT: usize = 40;

#[derive(Resource)]
pub(super) struct SelectionExportState {
    pub status_text: String,
    export_dir: PathBuf,
}

impl Default for SelectionExportState {
    fn default() -> Self {
        Self {
            status_text: "Export: idle".to_string(),
            export_dir: default_export_dir(),
        }
    }
}

#[derive(Component)]
pub(super) struct SelectionExportButton;

#[derive(Component)]
pub(super) struct SelectionExportStatusText;

#[derive(Debug, Clone, Serialize)]
struct SelectionExportMeta {
    kind: String,
    id: String,
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SelectionExportDocument {
    format_version: u8,
    exported_at_unix_ms: u128,
    snapshot_time: Option<u64>,
    selection: SelectionExportMeta,
    details_text: String,
    selection_state: Value,
    related_events: Vec<WorldEvent>,
    decision_traces: Vec<AgentDecisionTrace>,
}

pub(super) fn spawn_selection_export_controls(
    parent: &mut ChildSpawnerCommands,
    font: Handle<Font>,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            row_gap: Val::Px(4.0),
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
            .with_children(|row| {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        height: Val::Px(22.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.24, 0.3, 0.22)),
                    SelectionExportButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Export Selection"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });

            root.spawn((
                Text::new("Export: idle"),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.84, 0.76)),
                SelectionExportStatusText,
            ));
        });
}

pub(super) fn handle_selection_export_button(
    mut export_state: ResMut<SelectionExportState>,
    selection: Res<ViewerSelection>,
    state: Res<ViewerState>,
    mut interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<SelectionExportButton>,
        ),
    >,
) {
    for interaction in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match write_selection_export_file(&selection, &state, export_state.export_dir.as_path()) {
            Ok(path) => {
                export_state.status_text = format!("Export: saved {}", path.display());
            }
            Err(err) => {
                export_state.status_text = format!("Export: {err}");
            }
        }
    }
}

pub(super) fn update_selection_export_status_text(
    export_state: Res<SelectionExportState>,
    mut query: Query<&mut Text, With<SelectionExportStatusText>>,
) {
    if !export_state.is_changed() {
        return;
    }

    if let Ok(mut text) = query.single_mut() {
        text.0 = export_state.status_text.clone();
    }
}

fn write_selection_export_file(
    selection: &ViewerSelection,
    state: &ViewerState,
    export_dir: &Path,
) -> Result<PathBuf, String> {
    let export_doc = build_selection_export_document(selection, state)?;

    fs::create_dir_all(export_dir).map_err(|err| format!("create export dir failed: {err}"))?;

    let file_name = format!(
        "{}_{}-{}.json",
        export_doc.exported_at_unix_ms,
        export_doc.selection.kind,
        sanitize_file_segment(export_doc.selection.id.as_str()),
    );
    let path = export_dir.join(file_name);
    let content = serde_json::to_string_pretty(&export_doc)
        .map_err(|err| format!("serialize export doc failed: {err}"))?;
    fs::write(&path, content).map_err(|err| format!("write export file failed: {err}"))?;

    Ok(path)
}

fn build_selection_export_document(
    selection: &ViewerSelection,
    state: &ViewerState,
) -> Result<SelectionExportDocument, String> {
    let selected = selection
        .current
        .as_ref()
        .ok_or_else(|| "no selection".to_string())?;
    let snapshot = state
        .snapshot
        .as_ref()
        .ok_or_else(|| "no snapshot".to_string())?;

    let selection_state = selected_state_value(selected, snapshot)?;
    let related_events = collect_related_events(selected, snapshot, &state.events);
    let decision_traces = collect_related_traces(selected, snapshot, &state.decision_traces);

    let details_text = selection_details_summary(
        selection,
        state.snapshot.as_ref(),
        &state.events,
        &state.decision_traces,
    );

    Ok(SelectionExportDocument {
        format_version: 1,
        exported_at_unix_ms: now_unix_ms(),
        snapshot_time: state.snapshot.as_ref().map(|snapshot| snapshot.time),
        selection: SelectionExportMeta {
            kind: selection_kind_name(selected.kind).to_string(),
            id: selected.id.clone(),
            name: selected.name.clone(),
        },
        details_text,
        selection_state,
        related_events,
        decision_traces,
    })
}

fn selected_state_value(
    selected: &SelectionInfo,
    snapshot: &WorldSnapshot,
) -> Result<Value, String> {
    match selected.kind {
        SelectionKind::Agent => snapshot
            .model
            .agents
            .get(selected.id.as_str())
            .ok_or_else(|| format!("agent {} not found", selected.id))
            .and_then(to_json_value),
        SelectionKind::Location => snapshot
            .model
            .locations
            .get(selected.id.as_str())
            .ok_or_else(|| format!("location {} not found", selected.id))
            .and_then(to_json_value),
        SelectionKind::Asset => snapshot
            .model
            .assets
            .get(selected.id.as_str())
            .ok_or_else(|| format!("asset {} not found", selected.id))
            .and_then(to_json_value),
        SelectionKind::PowerPlant => snapshot
            .model
            .power_plants
            .get(selected.id.as_str())
            .ok_or_else(|| format!("power_plant {} not found", selected.id))
            .and_then(to_json_value),
        SelectionKind::PowerStorage => snapshot
            .model
            .power_storages
            .get(selected.id.as_str())
            .ok_or_else(|| format!("power_storage {} not found", selected.id))
            .and_then(to_json_value),
        SelectionKind::Chunk => chunk_state_value(selected.id.as_str(), snapshot),
    }
}

fn chunk_state_value(chunk_id: &str, snapshot: &WorldSnapshot) -> Result<Value, String> {
    let coord =
        parse_chunk_coord(chunk_id).ok_or_else(|| format!("invalid chunk id {chunk_id}"))?;
    let chunk_state = snapshot
        .model
        .chunks
        .get(&coord)
        .ok_or_else(|| format!("chunk {chunk_id} not found"))?;
    let bounds = chunk_bounds(coord, &snapshot.config.space).map(|bounds| {
        json!({
            "min_cm": {
                "x": bounds.min.x_cm,
                "y": bounds.min.y_cm,
                "z": bounds.min.z_cm,
            },
            "max_cm": {
                "x": bounds.max.x_cm,
                "y": bounds.max.y_cm,
                "z": bounds.max.z_cm,
            }
        })
    });

    Ok(json!({
        "coord": {
            "x": coord.x,
            "y": coord.y,
            "z": coord.z,
        },
        "state": chunk_state,
        "bounds": bounds,
    }))
}

fn collect_related_events(
    selected: &SelectionInfo,
    snapshot: &WorldSnapshot,
    events: &[WorldEvent],
) -> Vec<WorldEvent> {
    let mut filtered: Vec<WorldEvent> = events
        .iter()
        .filter(|event| event_matches_selection(event, selected, snapshot))
        .rev()
        .take(RELATED_EVENT_LIMIT)
        .cloned()
        .collect();
    filtered.reverse();
    filtered
}

fn collect_related_traces(
    selected: &SelectionInfo,
    snapshot: &WorldSnapshot,
    traces: &[AgentDecisionTrace],
) -> Vec<AgentDecisionTrace> {
    match selected.kind {
        SelectionKind::Agent => traces
            .iter()
            .filter(|trace| trace.agent_id == selected.id)
            .cloned()
            .collect(),
        SelectionKind::Asset => snapshot
            .model
            .assets
            .get(selected.id.as_str())
            .map(|asset| match &asset.owner {
                ResourceOwner::Agent { agent_id } => traces
                    .iter()
                    .filter(|trace| trace.agent_id == *agent_id)
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn event_matches_selection(
    event: &WorldEvent,
    selected: &SelectionInfo,
    snapshot: &WorldSnapshot,
) -> bool {
    match selected.kind {
        SelectionKind::Agent => event_matches_agent(event, selected.id.as_str()),
        SelectionKind::Location => event_matches_location(event, selected.id.as_str()),
        SelectionKind::Asset => snapshot
            .model
            .assets
            .get(selected.id.as_str())
            .map(|asset| event_matches_owner(event, &asset.owner))
            .unwrap_or(false),
        SelectionKind::PowerPlant => event_matches_power_plant(event, selected.id.as_str()),
        SelectionKind::PowerStorage => event_matches_power_storage(event, selected.id.as_str()),
        SelectionKind::Chunk => parse_chunk_coord(selected.id.as_str())
            .map(|coord| event_matches_chunk(event, coord))
            .unwrap_or(false),
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
        WorldEventKind::Power(power_event) => match power_event {
            PowerEvent::PowerConsumed { agent_id: id, .. }
            | PowerEvent::PowerCharged { agent_id: id, .. }
            | PowerEvent::PowerStateChanged { agent_id: id, .. } => id == agent_id,
            _ => false,
        },
        _ => false,
    }
}

fn event_matches_location(event: &WorldEvent, location_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::LocationRegistered {
            location_id: id, ..
        }
        | WorldEventKind::AgentRegistered {
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

fn event_matches_power_plant(event: &WorldEvent, plant_id: &str) -> bool {
    matches!(
        &event.kind,
        WorldEventKind::Power(PowerEvent::PowerPlantRegistered { plant }) if plant.id == plant_id
    ) || matches!(
        &event.kind,
        WorldEventKind::Power(PowerEvent::PowerGenerated { plant_id: id, .. }) if id == plant_id
    )
}

fn event_matches_power_storage(event: &WorldEvent, storage_id: &str) -> bool {
    matches!(
        &event.kind,
        WorldEventKind::Power(PowerEvent::PowerStorageRegistered { storage }) if storage.id == storage_id
    ) || matches!(
        &event.kind,
        WorldEventKind::Power(PowerEvent::PowerStored { storage_id: id, .. }) if id == storage_id
    ) || matches!(
        &event.kind,
        WorldEventKind::Power(PowerEvent::PowerDischarged { storage_id: id, .. }) if id == storage_id
    )
}

fn event_matches_chunk(event: &WorldEvent, coord: ChunkCoord) -> bool {
    matches!(
        &event.kind,
        WorldEventKind::ChunkGenerated { coord: event_coord, .. } if *event_coord == coord
    )
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
        _ => false,
    }
}

fn owner_is_agent(owner: &ResourceOwner, agent_id: &str) -> bool {
    matches!(
        owner,
        ResourceOwner::Agent { agent_id: id } if id == agent_id
    )
}

fn owner_is_location(owner: &ResourceOwner, location_id: &str) -> bool {
    matches!(
        owner,
        ResourceOwner::Location { location_id: id } if id == location_id
    )
}

fn parse_chunk_coord(id: &str) -> Option<ChunkCoord> {
    let mut parts = id.split(',');
    let x = parts.next()?.parse::<i32>().ok()?;
    let y = parts.next()?.parse::<i32>().ok()?;
    let z = parts.next()?.parse::<i32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(ChunkCoord { x, y, z })
}

fn to_json_value<T: Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|err| format!("serialize selection state failed: {err}"))
}

fn default_export_dir() -> PathBuf {
    std::env::var("AGENT_WORLD_VIEWER_EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".tmp/selection-exports"))
}

fn selection_kind_name(kind: SelectionKind) -> &'static str {
    match kind {
        SelectionKind::Agent => "agent",
        SelectionKind::Location => "location",
        SelectionKind::Asset => "asset",
        SelectionKind::PowerPlant => "power_plant",
        SelectionKind::PowerStorage => "power_storage",
        SelectionKind::Chunk => "chunk",
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn sanitize_file_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "selection".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        Agent, ChunkRuntimeConfig, Location, ResourceKind, RunnerMetrics, WorldConfig, WorldModel,
    };

    fn sample_snapshot() -> WorldSnapshot {
        let mut model = WorldModel::default();

        model.locations.insert(
            "loc-1".to_string(),
            Location::new("loc-1", "Alpha", GeoPos::new(0.0, 0.0, 0.0)),
        );
        model.locations.insert(
            "loc-2".to_string(),
            Location::new("loc-2", "Beta", GeoPos::new(100.0, 0.0, 0.0)),
        );

        let mut agent = Agent::new("agent-1", "loc-1", GeoPos::new(0.0, 0.0, 0.0));
        agent
            .resources
            .set(ResourceKind::Electricity, 42)
            .expect("set electricity");
        model.agents.insert("agent-1".to_string(), agent);

        WorldSnapshot {
            version: agent_world::simulator::SNAPSHOT_VERSION,
            chunk_generation_schema_version:
                agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
            time: 12,
            config: WorldConfig::default(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: 3,
            next_action_id: 3,
            pending_actions: Vec::new(),
            journal_len: 0,
        }
    }

    fn sample_selection() -> ViewerSelection {
        ViewerSelection {
            current: Some(SelectionInfo {
                entity: Entity::from_bits(1),
                kind: SelectionKind::Agent,
                id: "agent-1".to_string(),
                name: None,
            }),
        }
    }

    #[test]
    fn build_selection_export_document_contains_agent_state_events_and_traces() {
        let snapshot = sample_snapshot();
        let events = vec![
            WorldEvent {
                id: 1,
                time: 10,
                kind: WorldEventKind::AgentMoved {
                    agent_id: "agent-1".to_string(),
                    from: "loc-1".to_string(),
                    to: "loc-2".to_string(),
                    distance_cm: 100,
                    electricity_cost: 2,
                },
            },
            WorldEvent {
                id: 2,
                time: 11,
                kind: WorldEventKind::AgentMoved {
                    agent_id: "agent-2".to_string(),
                    from: "loc-1".to_string(),
                    to: "loc-2".to_string(),
                    distance_cm: 100,
                    electricity_cost: 2,
                },
            },
        ];

        let traces = vec![
            AgentDecisionTrace {
                time: 11,
                agent_id: "agent-1".to_string(),
                decision: agent_world::simulator::AgentDecision::Wait,
                llm_input: Some("input-a".to_string()),
                llm_output: Some("output-a".to_string()),
                llm_error: None,
                parse_error: None,
                llm_diagnostics: Some(agent_world::simulator::LlmDecisionDiagnostics {
                    model: Some("gpt".to_string()),
                    latency_ms: Some(12),
                    prompt_tokens: Some(7),
                    completion_tokens: Some(9),
                    total_tokens: Some(16),
                    retry_count: 0,
                }),
            },
            AgentDecisionTrace {
                time: 11,
                agent_id: "agent-2".to_string(),
                decision: agent_world::simulator::AgentDecision::Wait,
                llm_input: Some("input-b".to_string()),
                llm_output: Some("output-b".to_string()),
                llm_error: None,
                parse_error: None,
                llm_diagnostics: Some(agent_world::simulator::LlmDecisionDiagnostics {
                    model: Some("gpt".to_string()),
                    latency_ms: Some(10),
                    prompt_tokens: Some(6),
                    completion_tokens: Some(8),
                    total_tokens: Some(14),
                    retry_count: 0,
                }),
            },
        ];

        let state = ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: Some(snapshot),
            events,
            decision_traces: traces,
            metrics: Some(RunnerMetrics::default()),
        };

        let document = build_selection_export_document(&sample_selection(), &state)
            .expect("build selection export doc");

        assert_eq!(document.format_version, 1);
        assert_eq!(document.snapshot_time, Some(12));
        assert_eq!(document.selection.kind, "agent");
        assert_eq!(document.selection.id, "agent-1");
        assert!(document.details_text.contains("Details: agent agent-1"));
        assert_eq!(document.related_events.len(), 1);
        assert_eq!(document.related_events[0].id, 1);
        assert_eq!(document.decision_traces.len(), 1);
        assert_eq!(document.decision_traces[0].agent_id, "agent-1");
    }

    #[test]
    fn handle_selection_export_button_sets_error_without_selection() {
        let mut app = App::new();
        app.add_systems(Update, handle_selection_export_button);
        app.world_mut()
            .insert_resource(SelectionExportState::default());
        app.world_mut().insert_resource(ViewerSelection::default());
        app.world_mut().insert_resource(ViewerState::default());
        app.world_mut()
            .spawn((Button, Interaction::Pressed, SelectionExportButton));

        app.update();

        let export_state = app.world().resource::<SelectionExportState>();
        assert!(export_state.status_text.contains("no selection"));
    }

    #[test]
    fn write_selection_export_file_creates_json() {
        let snapshot = sample_snapshot();
        let state = ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: Some(snapshot),
            events: Vec::new(),
            decision_traces: Vec::new(),
            metrics: None,
        };

        let selection = sample_selection();
        let export_dir = std::env::temp_dir().join(format!("agent-world-export-{}", now_unix_ms()));
        let path = write_selection_export_file(&selection, &state, export_dir.as_path())
            .expect("write export file");

        assert!(path.exists());
        let content = fs::read_to_string(&path).expect("read export content");
        assert!(content.contains("\"selection\""));
        assert!(content.contains("\"agent-1\""));

        let _ = fs::remove_file(path);
        let _ = fs::remove_dir_all(export_dir);
    }
}
