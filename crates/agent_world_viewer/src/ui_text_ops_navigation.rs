use std::collections::BTreeMap;

use agent_world::simulator::{
    chunk_coord_of, ChunkCoord, PowerEvent, RejectReason, ResourceOwner, WorldEvent,
    WorldEventKind, WorldSnapshot,
};

const OPS_NAV_EVENT_WINDOW: usize = 160;
const OPS_NAV_TOP_LIMIT: usize = 3;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ChunkHotspot {
    events: usize,
    alerts: usize,
}

pub(super) fn ops_navigation_alert_summary(
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> Option<String> {
    let snapshot = snapshot?;

    let mut chunk_hotspots = BTreeMap::<ChunkCoord, ChunkHotspot>::new();
    let mut node_hotspots = BTreeMap::<String, usize>::new();
    let mut root_causes = BTreeMap::<String, usize>::new();
    let mut total_events = 0_usize;
    let mut alert_events = 0_usize;

    for event in events.iter().rev().take(OPS_NAV_EVENT_WINDOW) {
        total_events += 1;
        let is_alert = matches!(event.kind, WorldEventKind::ActionRejected { .. });
        if is_alert {
            alert_events += 1;
        }

        if let Some(coord) = chunk_coord_for_event(snapshot, event) {
            let entry = chunk_hotspots.entry(coord).or_default();
            entry.events += 1;
            if is_alert {
                entry.alerts += 1;
            }
        }

        for node in node_targets_for_event(event) {
            let entry = node_hotspots.entry(node).or_insert(0);
            *entry += 1;
        }

        if let WorldEventKind::ActionRejected { reason } = &event.kind {
            let cause = root_cause_key(reason);
            let entry = root_causes.entry(cause).or_insert(0);
            *entry += 1;
        }
    }

    if total_events == 0 {
        return None;
    }

    let mut lines = vec!["Ops Navigator:".to_string()];
    lines.push("World:".to_string());
    lines.push(format!("- Activity Events(Recent): {total_events}"));
    lines.push(format!("- Alert Events(Recent): {alert_events}"));

    lines.push("".to_string());
    lines.push("Region Hotspots:".to_string());
    let mut hotspot_entries: Vec<_> = chunk_hotspots.into_iter().collect();
    hotspot_entries.sort_by(|left, right| {
        right
            .1
            .alerts
            .cmp(&left.1.alerts)
            .then_with(|| right.1.events.cmp(&left.1.events))
            .then_with(|| left.0.cmp(&right.0))
    });
    if hotspot_entries.is_empty() {
        lines.push("- (none)".to_string());
    } else {
        for (coord, hotspot) in hotspot_entries.into_iter().take(OPS_NAV_TOP_LIMIT) {
            lines.push(format!(
                "- chunk({}, {}, {}): events={} alerts={}",
                coord.x, coord.y, coord.z, hotspot.events, hotspot.alerts
            ));
        }
    }

    lines.push("".to_string());
    lines.push("Node Hotspots:".to_string());
    let mut node_entries: Vec<_> = node_hotspots.into_iter().collect();
    node_entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    if node_entries.is_empty() {
        lines.push("- (none)".to_string());
    } else {
        for (node, score) in node_entries.into_iter().take(OPS_NAV_TOP_LIMIT) {
            lines.push(format!("- {node}: score={score}"));
        }
    }

    lines.push("".to_string());
    lines.push("Alert Root Causes:".to_string());
    let mut cause_entries: Vec<_> = root_causes.into_iter().collect();
    cause_entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    if cause_entries.is_empty() {
        lines.push("- (none)".to_string());
    } else {
        for (cause, count) in cause_entries.into_iter().take(OPS_NAV_TOP_LIMIT) {
            lines.push(format!("- {cause}: {count}"));
        }
    }

    Some(lines.join("\n"))
}

fn root_cause_key(reason: &RejectReason) -> String {
    let raw = format!("{reason:?}");
    raw.split('{')
        .next()
        .unwrap_or(raw.as_str())
        .trim()
        .to_string()
}

fn chunk_coord_for_event(snapshot: &WorldSnapshot, event: &WorldEvent) -> Option<ChunkCoord> {
    if let WorldEventKind::ChunkGenerated { coord, .. } = event.kind {
        return Some(coord);
    }

    let location_id = location_target_for_event(event)?;
    let location = snapshot.model.locations.get(location_id.as_str())?;
    chunk_coord_of(location.pos, &snapshot.config.space)
}

fn location_target_for_event(event: &WorldEvent) -> Option<String> {
    match &event.kind {
        WorldEventKind::LocationRegistered { location_id, .. } => Some(location_id.clone()),
        WorldEventKind::AgentRegistered { location_id, .. } => Some(location_id.clone()),
        WorldEventKind::AgentMoved { to, .. } => Some(to.clone()),
        WorldEventKind::RadiationHarvested { location_id, .. } => Some(location_id.clone()),
        WorldEventKind::Power(PowerEvent::PowerGenerated { location_id, .. }) => {
            Some(location_id.clone())
        }
        WorldEventKind::Power(PowerEvent::PowerStored { location_id, .. }) => {
            Some(location_id.clone())
        }
        WorldEventKind::Power(PowerEvent::PowerDischarged { location_id, .. }) => {
            Some(location_id.clone())
        }
        WorldEventKind::ResourceTransferred { from, to, .. } => {
            owner_location_id(from).or_else(|| owner_location_id(to))
        }
        WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            owner_location_id(from).or_else(|| owner_location_id(to))
        }
        WorldEventKind::ActionRejected { reason } => match reason {
            RejectReason::LocationNotFound { location_id } => Some(location_id.clone()),
            RejectReason::AgentNotAtLocation { location_id, .. } => Some(location_id.clone()),
            RejectReason::AgentAlreadyAtLocation { location_id, .. } => Some(location_id.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn node_targets_for_event(event: &WorldEvent) -> Vec<String> {
    match &event.kind {
        WorldEventKind::AgentRegistered { agent_id, .. } => vec![format!("agent::{agent_id}")],
        WorldEventKind::AgentMoved {
            agent_id, from, to, ..
        } => vec![
            format!("agent::{agent_id}"),
            format!("location::{from}"),
            format!("location::{to}"),
        ],
        WorldEventKind::RadiationHarvested {
            agent_id,
            location_id,
            ..
        } => vec![
            format!("agent::{agent_id}"),
            format!("location::{location_id}"),
        ],
        WorldEventKind::ResourceTransferred { from, to, .. } => {
            vec![owner_label(from), owner_label(to)]
        }
        WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            vec![owner_label(from), owner_label(to)]
        }
        WorldEventKind::ActionRejected { reason } => match reason {
            RejectReason::AgentNotFound { agent_id } => vec![format!("agent::{agent_id}")],
            RejectReason::AgentNotAtLocation {
                agent_id,
                location_id,
            } => vec![
                format!("agent::{agent_id}"),
                format!("location::{location_id}"),
            ],
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn owner_location_id(owner: &ResourceOwner) -> Option<String> {
    match owner {
        ResourceOwner::Location { location_id } => Some(location_id.clone()),
        _ => None,
    }
}

fn owner_label(owner: &ResourceOwner) -> String {
    match owner {
        ResourceOwner::Agent { agent_id } => format!("agent::{agent_id}"),
        ResourceOwner::Location { location_id } => format!("location::{location_id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        ChunkRuntimeConfig, Location, ResourceKind, WorldConfig, WorldModel, WorldSnapshot,
        CHUNK_GENERATION_SCHEMA_VERSION, SNAPSHOT_VERSION,
    };

    #[test]
    fn ops_navigation_alert_summary_returns_none_without_snapshot() {
        assert!(ops_navigation_alert_summary(None, &[]).is_none());
    }

    #[test]
    fn ops_navigation_alert_summary_reports_regions_nodes_and_causes() {
        let mut model = WorldModel::default();
        model.locations.insert(
            "loc-a".to_string(),
            Location::new("loc-a", "Alpha", GeoPos::new(1.0, 1.0, 1.0)),
        );
        model.locations.insert(
            "loc-b".to_string(),
            Location::new("loc-b", "Beta", GeoPos::new(2_100_000.0, 1.0, 1.0)),
        );

        let snapshot = WorldSnapshot {
            version: SNAPSHOT_VERSION,
            chunk_generation_schema_version: CHUNK_GENERATION_SCHEMA_VERSION,
            time: 42,
            config: WorldConfig::default(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: 6,
            next_action_id: 3,
            pending_actions: Vec::new(),
            journal_len: 5,
        };

        let events = vec![
            WorldEvent {
                id: 1,
                time: 31,
                kind: WorldEventKind::ResourceTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    to: ResourceOwner::Location {
                        location_id: "loc-b".to_string(),
                    },
                    kind: ResourceKind::Hardware,
                    amount: 2,
                },
            },
            WorldEvent {
                id: 2,
                time: 32,
                kind: WorldEventKind::AgentMoved {
                    agent_id: "agent-1".to_string(),
                    from: "loc-a".to_string(),
                    to: "loc-b".to_string(),
                    distance_cm: 100,
                    electricity_cost: 1,
                },
            },
            WorldEvent {
                id: 3,
                time: 33,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InsufficientResource {
                        owner: ResourceOwner::Agent {
                            agent_id: "agent-1".to_string(),
                        },
                        kind: ResourceKind::Data,
                        requested: 7,
                        available: 2,
                    },
                },
            },
        ];

        let summary =
            ops_navigation_alert_summary(Some(&snapshot), &events).expect("summary should exist");
        assert!(summary.contains("Ops Navigator:"));
        assert!(summary.contains("World:"));
        assert!(summary.contains("Activity Events(Recent): 3"));
        assert!(summary.contains("Alert Events(Recent): 1"));
        assert!(summary.contains("Region Hotspots:"));
        assert!(summary.contains("Node Hotspots:"));
        assert!(summary.contains("Alert Root Causes:"));
        assert!(summary.contains("InsufficientResource"));
    }
}
