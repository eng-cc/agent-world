use agent_world::geometry::GeoPos;
use agent_world::simulator::{
    chunk_bounds, AgentDecisionTrace, Asset, AssetKind, ChunkCoord, ChunkState,
    FragmentElementKind, ModuleVisualAnchor, PowerEvent, PowerPlant, ResourceKind, ResourceOwner,
    RunnerMetrics, WorldEvent, WorldEventKind, WorldSnapshot,
};

use super::viewer_3d_config::ViewerPhysicalRenderConfig;
use super::{ConnectionStatus, SelectionKind, ViewerSelection};

pub(super) fn format_status(status: &ConnectionStatus) -> String {
    match status {
        ConnectionStatus::Connecting => "connecting".to_string(),
        ConnectionStatus::Connected => "connected".to_string(),
        ConnectionStatus::Error(message) => format!("error: {message}"),
    }
}

pub(super) fn world_summary(
    snapshot: Option<&WorldSnapshot>,
    metrics: Option<&RunnerMetrics>,
    physical: Option<&ViewerPhysicalRenderConfig>,
) -> String {
    let mut lines = Vec::new();
    if let Some(snapshot) = snapshot {
        let model = &snapshot.model;
        lines.push(format!("Time: {}", snapshot.time));
        lines.push(format!("Locations: {}", model.locations.len()));
        lines.push(format!("Agents: {}", model.agents.len()));
        lines.push(format!("Assets: {}", model.assets.len()));
        lines.push(format!(
            "Module Visuals: {}",
            model.module_visual_entities.len()
        ));
        lines.push(format!("Power Plants: {}", model.power_plants.len()));
        lines.push(format!("Power Storages: {}", model.power_storages.len()));
        lines.push(format!("Chunks: {}", model.chunks.len()));
    } else {
        lines.push("World: (no snapshot)".to_string());
    }

    if let Some(metrics) = metrics {
        lines.push("".to_string());
        lines.push(format!("Ticks: {}", metrics.total_ticks));
        lines.push(format!("Actions: {}", metrics.total_actions));
        lines.push(format!("Decisions: {}", metrics.total_decisions));
    }

    if let Some(physical) = physical {
        lines.push("".to_string());
        lines.push(format!(
            "Render Physical: {}",
            if physical.enabled { "on" } else { "off" }
        ));
        if physical.enabled {
            lines.push(format!("Unit: 1u={:.2}m", physical.meters_per_unit));
            lines.push(format!(
                "Camera Clip(m): near={:.2} far={:.0}",
                physical.camera_near_m, physical.camera_far_m
            ));
            lines.push(format!(
                "Stellar Distance(AU): {:.2}",
                physical.stellar_distance_au
            ));
            lines.push(format!(
                "Irradiance(W/m²): {:.1}",
                physical.irradiance_w_m2()
            ));
            lines.push(format!(
                "Exposed Illuminance(lux): {:.0}",
                physical.exposed_illuminance_lux()
            ));
            lines.push(format!("Exposure(EV100): {:.2}", physical.exposure_ev100));
            lines.push(format!(
                "Radiation Ref Area(m²): {:.2}",
                physical.reference_radiation_area_m2
            ));
        }
    }

    lines.join("\n")
}

pub(super) fn events_summary(events: &[WorldEvent], focus_tick: Option<u64>) -> String {
    const WINDOW_SIZE: usize = 20;

    if events.is_empty() {
        return "Events:
(no events)"
            .to_string();
    }

    if focus_tick.is_none() {
        let mut lines = Vec::new();
        lines.push("Events:".to_string());
        for event in events.iter().rev().take(WINDOW_SIZE).rev() {
            lines.push(format!("#{} t{} {:?}", event.id, event.time, event.kind));
        }
        return lines.join("\n");
    }

    let requested_focus = focus_tick.unwrap_or(0);
    let mut nearest_idx = 0_usize;
    let mut nearest_dist = u64::MAX;

    for (idx, event) in events.iter().enumerate() {
        let dist = event.time.abs_diff(requested_focus);
        if dist < nearest_dist {
            nearest_dist = dist;
            nearest_idx = idx;
        }
    }

    let total = events.len();
    let half = WINDOW_SIZE / 2;
    let max_start = total.saturating_sub(WINDOW_SIZE);
    let window_start = nearest_idx.saturating_sub(half).min(max_start);
    let window_end = (window_start + WINDOW_SIZE).min(total);

    let focused = &events[nearest_idx];
    let mut lines = Vec::new();
    lines.push("Events (focused):".to_string());
    lines.push(format!(
        "Focus: requested t{} -> nearest t{} (#{}), Δt={}",
        requested_focus, focused.time, focused.id, nearest_dist
    ));
    for (idx, event) in events
        .iter()
        .enumerate()
        .skip(window_start)
        .take(window_end - window_start)
    {
        let prefix = if idx == nearest_idx { ">>" } else { "  " };
        lines.push(format!(
            "{} #{} t{} {:?}",
            prefix, event.id, event.time, event.kind
        ));
    }
    lines.join("\n")
}

pub(super) fn agent_activity_summary(
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> String {
    let Some(snapshot) = snapshot else {
        return "Agents Activity:\n(no snapshot)".to_string();
    };

    if snapshot.model.agents.is_empty() {
        return "Agents Activity:\n(none)".to_string();
    }

    let mut lines = Vec::new();
    lines.push("Agents Activity:".to_string());

    let mut agent_ids: Vec<_> = snapshot.model.agents.keys().cloned().collect();
    agent_ids.sort();

    for agent_id in agent_ids {
        if let Some(agent) = snapshot.model.agents.get(&agent_id) {
            let electricity = agent.resources.get(ResourceKind::Electricity);
            let activity =
                latest_agent_activity(&agent_id, events).unwrap_or_else(|| "idle".to_string());
            lines.push(format!(
                "{agent_id} @ {} | E={} | {}",
                agent.location_id, electricity, activity
            ));
        }
    }

    lines.join("\n")
}

pub(super) fn selection_details_summary(
    selection: &ViewerSelection,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
    decision_traces: &[AgentDecisionTrace],
    reference_radiation_area_m2: f32,
) -> String {
    let Some(selected) = selection.current.as_ref() else {
        return "Details:\n(click object to inspect)".to_string();
    };

    match selected.kind {
        SelectionKind::Agent => {
            agent_details_summary(selected.id.as_str(), snapshot, events, decision_traces)
        }
        SelectionKind::Location => location_details_summary(
            selected.id.as_str(),
            selected.name.as_deref(),
            snapshot,
            events,
            reference_radiation_area_m2,
        ),
        SelectionKind::Asset => asset_details_summary(selected.id.as_str(), snapshot, events),
        SelectionKind::PowerPlant => {
            power_plant_details_summary(selected.id.as_str(), snapshot, events)
        }
        SelectionKind::PowerStorage => {
            power_storage_details_summary(selected.id.as_str(), snapshot, events)
        }
        SelectionKind::Chunk => chunk_details_summary(
            selected.id.as_str(),
            selected.name.as_deref(),
            snapshot,
            events,
        ),
    }
}

fn agent_details_summary(
    agent_id: &str,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
    decision_traces: &[AgentDecisionTrace],
) -> String {
    let Some(snapshot) = snapshot else {
        return format!("Details: agent {agent_id}\n(no snapshot)");
    };

    let Some(agent) = snapshot.model.agents.get(agent_id) else {
        return format!("Details: agent {agent_id}\n(not found in snapshot)");
    };

    let mut lines = Vec::new();
    lines.push(format!("Details: agent {agent_id}"));
    lines.push(format!("Location: {}", agent.location_id));
    lines.push(format!("Pos(cm): {}", format_geo_pos(agent.pos)));
    lines.push(format!(
        "Body: kind={} height={}cm",
        agent.body.kind, agent.body.height_cm
    ));
    lines.push(format!(
        "Power: {}/{} ({:?})",
        agent.power.level, agent.power.capacity, agent.power.state
    ));
    lines.push(format!("Thermal: heat={}", agent.thermal.heat));
    let thermal_ratio = thermal_ratio(agent.thermal.heat, snapshot.config.physics.thermal_capacity);
    lines.push(format!(
        "Thermal Visual: ratio={:.2} color={}",
        thermal_ratio,
        thermal_ratio_color(thermal_ratio)
    ));

    lines.push("Resources:".to_string());
    lines.extend(format_resource_stock(&agent.resources.amounts));

    lines.push("".to_string());
    lines.push("Recent Events:".to_string());
    let mut recent_events = agent_recent_events(agent_id, events, 6);
    if recent_events.is_empty() {
        recent_events.push("(none)".to_string());
    }
    lines.extend(recent_events);

    lines.push("".to_string());
    lines.push("Recent LLM I/O:".to_string());
    let mut recent_traces = agent_recent_traces(agent_id, decision_traces, 3);
    if recent_traces.is_empty() {
        recent_traces.push("(no llm trace yet)".to_string());
    }
    lines.extend(recent_traces);

    lines.join("\n")
}

fn location_details_summary(
    location_id: &str,
    selected_name: Option<&str>,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
    reference_radiation_area_m2: f32,
) -> String {
    let Some(snapshot) = snapshot else {
        return format!("Details: location {location_id}\n(no snapshot)");
    };

    let Some(location) = snapshot.model.locations.get(location_id) else {
        return format!("Details: location {location_id}\n(not found in snapshot)");
    };

    let plant_count = snapshot
        .model
        .power_plants
        .values()
        .filter(|plant| plant.location_id == location_id)
        .count();
    let storage_count = snapshot
        .model
        .power_storages
        .values()
        .filter(|storage| storage.location_id == location_id)
        .count();
    let asset_count = snapshot
        .model
        .assets
        .values()
        .filter(|asset| owner_matches_location(&asset.owner, location_id))
        .count();

    let mut lines = Vec::new();
    lines.push(format!("Details: location {location_id}"));
    lines.push(format!(
        "Name: {}",
        selected_name.unwrap_or(location.name.as_str())
    ));
    lines.push(format!("Pos(cm): {}", format_geo_pos(location.pos)));
    lines.push(format!(
        "Profile: material={:?} radius={}cm radiation/tick={}",
        location.profile.material,
        location.profile.radius_cm,
        location.profile.radiation_emission_per_tick
    ));
    let (radiation_power_w, radiation_flux_w_m2, area_m2) = radiation_visual_metrics(
        location.profile.radiation_emission_per_tick,
        snapshot.config.physics.power_unit_j,
        snapshot.config.physics.time_step_s,
        reference_radiation_area_m2,
    );
    lines.push(format!(
        "Radiation Visual: power={radiation_power_w:.2}W flux={radiation_flux_w_m2:.2}W/m2 area={area_m2:.2}m2"
    ));
    lines.push(format!(
        "Facilities: plants={} storages={} assets_owned={}",
        plant_count, storage_count, asset_count
    ));

    lines.push("Resources:".to_string());
    lines.extend(format_resource_stock(&location.resources.amounts));

    if let Some(fragment) = location.fragment_profile.as_ref() {
        lines.push("".to_string());
        lines.push(format!(
            "Fragment: blocks={} mass={}g density={}kg/m3",
            fragment.blocks.blocks.len(),
            fragment.total_mass_g,
            fragment.bulk_density_kg_per_m3
        ));
    }

    if let Some(budget) = location.fragment_budget.as_ref() {
        lines.push("Fragment Budget (remaining top):".to_string());
        let mut remaining: Vec<_> = budget.remaining_by_element_g.iter().collect();
        remaining.sort_by(|a, b| b.1.cmp(a.1));
        for (kind, amount) in remaining.into_iter().take(6) {
            lines.push(format!("- {:?}: {}g", kind, amount));
        }
    }

    lines.push("".to_string());
    lines.push("Recent Events:".to_string());
    let mut related = location_recent_events(location_id, events, 6);
    if related.is_empty() {
        related.push("(none)".to_string());
    }
    lines.extend(related);

    lines.join("\n")
}

fn asset_details_summary(
    asset_id: &str,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> String {
    let Some(snapshot) = snapshot else {
        return format!("Details: asset {asset_id}\n(no snapshot)");
    };

    if let Some(asset) = snapshot.model.assets.get(asset_id) {
        let mut lines = Vec::new();
        lines.push(format!("Details: asset {asset_id}"));
        lines.push(format!("Kind: {}", asset_kind_name(asset)));
        lines.push(format!("Quantity: {}", asset.quantity));
        lines.push(format!("Owner: {}", owner_label(&asset.owner)));
        if let Some(anchor) = owner_anchor_pos(snapshot, &asset.owner) {
            lines.push(format!("Owner Pos(cm): {}", format_geo_pos(anchor)));
        }

        lines.push("".to_string());
        lines.push("Recent Owner Events:".to_string());
        let mut related = owner_recent_events(&asset.owner, events, 6);
        if related.is_empty() {
            related.push("(none)".to_string());
        }
        lines.extend(related);

        return lines.join("\n");
    }

    if let Some(module_entity) = snapshot.model.module_visual_entities.get(asset_id) {
        return module_visual_details_summary(module_entity, snapshot, events);
    }

    format!("Details: asset {asset_id}\n(not found in snapshot)")
}

fn power_plant_details_summary(
    facility_id: &str,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> String {
    let Some(snapshot) = snapshot else {
        return format!("Details: power_plant {facility_id}\n(no snapshot)");
    };

    let Some(plant) = snapshot.model.power_plants.get(facility_id) else {
        return format!("Details: power_plant {facility_id}\n(not found in snapshot)");
    };

    facility_details_lines(facility_id, plant, snapshot, events).join("\n")
}

fn power_storage_details_summary(
    facility_id: &str,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> String {
    let Some(snapshot) = snapshot else {
        return format!("Details: power_storage {facility_id}\n(no snapshot)");
    };

    let Some(storage) = snapshot.model.power_storages.get(facility_id) else {
        return format!("Details: power_storage {facility_id}\n(not found in snapshot)");
    };

    let mut lines = Vec::new();
    lines.push(format!("Details: power_storage {facility_id}"));
    lines.push(format!("Location: {}", storage.location_id));
    lines.push(format!("Owner: {}", owner_label(&storage.owner)));
    lines.push(format!(
        "Level: {}/{} (charge_eff={:.2}, discharge_eff={:.2})",
        storage.current_level,
        storage.capacity,
        storage.charge_efficiency,
        storage.discharge_efficiency
    ));
    lines.push(format!(
        "Rates: max_charge={} max_discharge={}",
        storage.max_charge_rate, storage.max_discharge_rate
    ));

    if let Some(location) = snapshot.model.locations.get(&storage.location_id) {
        lines.push(format!(
            "Location Pos(cm): {}",
            format_geo_pos(location.pos)
        ));
    }

    lines.push("".to_string());
    lines.push("Recent Events:".to_string());
    let mut related = power_storage_recent_events(facility_id, events, 6);
    if related.is_empty() {
        related.push("(none)".to_string());
    }
    lines.extend(related);

    lines.join("\n")
}

fn chunk_details_summary(
    chunk_id: &str,
    selected_state: Option<&str>,
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> String {
    let Some(snapshot) = snapshot else {
        return format!("Details: chunk {chunk_id}\n(no snapshot)");
    };

    let Some(coord) = parse_chunk_coord(chunk_id) else {
        return format!("Details: chunk {chunk_id}\n(invalid chunk id)");
    };

    let Some(state) = snapshot.model.chunks.get(&coord) else {
        return format!("Details: chunk {chunk_id}\n(not found in snapshot)");
    };

    let mut lines = Vec::new();
    lines.push(format!("Details: chunk {chunk_id}"));
    lines.push(format!(
        "State: {}",
        selected_state.unwrap_or(chunk_state_name(*state))
    ));

    if let Some(bounds) = chunk_bounds(coord, &snapshot.config.space) {
        lines.push(format!(
            "Bounds(cm): x[{:.0},{:.0}] y[{:.0},{:.0}] z[{:.0},{:.0}]",
            bounds.min.x_cm,
            bounds.max.x_cm,
            bounds.min.y_cm,
            bounds.max.y_cm,
            bounds.min.z_cm,
            bounds.max.z_cm
        ));
    }

    let reservation_count = snapshot
        .model
        .chunk_boundary_reservations
        .get(&coord)
        .map(|items| items.len())
        .unwrap_or(0);
    lines.push(format!("Boundary Reservations: {}", reservation_count));

    lines.push("".to_string());
    lines.push("Budget (remaining top):".to_string());
    if let Some(budget) = snapshot.model.chunk_resource_budgets.get(&coord) {
        lines.extend(format_element_budget(&budget.remaining_by_element_g, 6));

        lines.push("Budget (total top):".to_string());
        lines.extend(format_element_budget(&budget.total_by_element_g, 6));
    } else {
        lines.push("- (none)".to_string());
    }

    lines.push("".to_string());
    lines.push("Recent Events:".to_string());
    let mut related = chunk_recent_events(coord, events, 6);
    if related.is_empty() {
        related.push("(none)".to_string());
    }
    lines.extend(related);

    lines.join("\n")
}

fn parse_chunk_coord(chunk_id: &str) -> Option<ChunkCoord> {
    let mut parts = chunk_id.split(',');
    let x = parts.next()?.trim().parse::<i32>().ok()?;
    let y = parts.next()?.trim().parse::<i32>().ok()?;
    let z = parts.next()?.trim().parse::<i32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(ChunkCoord { x, y, z })
}

fn chunk_state_name(state: ChunkState) -> &'static str {
    match state {
        ChunkState::Unexplored => "unexplored",
        ChunkState::Generated => "generated",
        ChunkState::Exhausted => "exhausted",
    }
}

fn format_element_budget(
    budgets: &std::collections::BTreeMap<FragmentElementKind, i64>,
    limit: usize,
) -> Vec<String> {
    if budgets.is_empty() {
        return vec!["- (empty)".to_string()];
    }
    let mut entries: Vec<_> = budgets.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));
    entries
        .into_iter()
        .take(limit)
        .map(|(kind, amount)| format!("- {:?}: {}g", kind, amount))
        .collect()
}

fn chunk_recent_events(coord: ChunkCoord, events: &[WorldEvent], limit: usize) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_chunk(event, coord)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn facility_details_lines(
    facility_id: &str,
    plant: &PowerPlant,
    snapshot: &WorldSnapshot,
    events: &[WorldEvent],
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Details: power_plant {facility_id}"));
    lines.push(format!("Location: {}", plant.location_id));
    lines.push(format!("Owner: {}", owner_label(&plant.owner)));
    lines.push(format!("Status: {:?}", plant.status));
    lines.push(format!(
        "Output: current={} capacity/tick={} effective={}",
        plant.current_output,
        plant.capacity_per_tick,
        plant.effective_output()
    ));
    lines.push(format!(
        "Costs: fuel_per_pu={} maintenance={} efficiency={:.2} degradation={:.2}",
        plant.fuel_cost_per_pu, plant.maintenance_cost, plant.efficiency, plant.degradation
    ));

    if let Some(location) = snapshot.model.locations.get(&plant.location_id) {
        lines.push(format!(
            "Location Pos(cm): {}",
            format_geo_pos(location.pos)
        ));
    }

    lines.push("".to_string());
    lines.push("Recent Events:".to_string());
    let mut related = power_plant_recent_events(facility_id, events, 6);
    if related.is_empty() {
        related.push("(none)".to_string());
    }
    lines.extend(related);

    lines
}

fn module_visual_details_summary(
    module_entity: &agent_world::simulator::ModuleVisualEntity,
    snapshot: &WorldSnapshot,
    events: &[WorldEvent],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Details: module_visual {}",
        module_entity.entity_id
    ));
    lines.push(format!("Module: {}", module_entity.module_id));
    lines.push(format!("Kind: {}", module_entity.kind));
    lines.push(format!(
        "Label: {}",
        module_entity
            .label
            .as_deref()
            .filter(|label| !label.trim().is_empty())
            .unwrap_or("(none)")
    ));
    lines.push(format!(
        "Anchor: {}",
        module_visual_anchor_label(&module_entity.anchor)
    ));
    if let Some(pos) = module_visual_anchor_pos(snapshot, &module_entity.anchor) {
        lines.push(format!("Anchor Pos(cm): {}", format_geo_pos(pos)));
    }

    lines.push("".to_string());
    lines.push("Recent Events:".to_string());
    let mut related = module_visual_recent_events(module_entity.entity_id.as_str(), events, 6);
    if related.is_empty() {
        related.push("(none)".to_string());
    }
    lines.extend(related);

    lines.join("\n")
}

fn module_visual_anchor_label(anchor: &ModuleVisualAnchor) -> String {
    match anchor {
        ModuleVisualAnchor::Agent { agent_id } => format!("agent::{agent_id}"),
        ModuleVisualAnchor::Location { location_id } => format!("location::{location_id}"),
        ModuleVisualAnchor::Absolute { pos } => format!("absolute({})", format_geo_pos(*pos)),
    }
}

fn module_visual_anchor_pos(
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

fn asset_kind_name(asset: &Asset) -> String {
    match &asset.kind {
        AssetKind::Resource { kind } => format!("resource::{kind:?}"),
    }
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

fn owner_label(owner: &ResourceOwner) -> String {
    match owner {
        ResourceOwner::Agent { agent_id } => format!("agent::{agent_id}"),
        ResourceOwner::Location { location_id } => format!("location::{location_id}"),
    }
}

fn format_geo_pos(pos: GeoPos) -> String {
    format!("x={:.0}, y={:.0}, z={:.0}", pos.x_cm, pos.y_cm, pos.z_cm)
}

fn thermal_ratio(heat: i64, capacity: i64) -> f64 {
    let heat = heat.max(0) as f64;
    let capacity = capacity.max(1) as f64;
    heat / capacity
}

fn thermal_ratio_color(thermal_ratio: f64) -> &'static str {
    if thermal_ratio <= 0.6 {
        "heat_low"
    } else if thermal_ratio <= 1.0 {
        "heat_mid"
    } else {
        "heat_high"
    }
}

fn radiation_visual_metrics(
    radiation_emission_per_tick: i64,
    power_unit_j: i64,
    time_step_s: i64,
    reference_radiation_area_m2: f32,
) -> (f64, f64, f64) {
    let emission = radiation_emission_per_tick.max(0) as f64;
    let joule_per_unit = power_unit_j.max(1) as f64;
    let seconds_per_tick = time_step_s.max(1) as f64;
    let area_m2 = if reference_radiation_area_m2.is_finite() && reference_radiation_area_m2 > 0.0 {
        reference_radiation_area_m2 as f64
    } else {
        1.0
    };
    let radiation_power_w = emission * joule_per_unit / seconds_per_tick;
    let radiation_flux_w_m2 = radiation_power_w / area_m2;
    (radiation_power_w, radiation_flux_w_m2, area_m2)
}

fn format_resource_stock(amounts: &std::collections::BTreeMap<ResourceKind, i64>) -> Vec<String> {
    if amounts.is_empty() {
        return vec!["- (empty)".to_string()];
    }
    amounts
        .iter()
        .map(|(kind, amount)| format!("- {:?}: {}", kind, amount))
        .collect()
}

fn agent_recent_events(agent_id: &str, events: &[WorldEvent], limit: usize) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_agent(event, agent_id)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn location_recent_events(location_id: &str, events: &[WorldEvent], limit: usize) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_location(event, location_id)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn owner_recent_events(owner: &ResourceOwner, events: &[WorldEvent], limit: usize) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_owner(event, owner)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn power_plant_recent_events(plant_id: &str, events: &[WorldEvent], limit: usize) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_power_plant(event, plant_id)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn power_storage_recent_events(
    storage_id: &str,
    events: &[WorldEvent],
    limit: usize,
) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_power_storage(event, storage_id)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn module_visual_recent_events(
    entity_id: &str,
    events: &[WorldEvent],
    limit: usize,
) -> Vec<String> {
    events
        .iter()
        .rev()
        .filter_map(|event| {
            event_activity_for_module_visual(event, entity_id)
                .map(|activity| format!("- t{} #{} {}", event.time, event.id, activity))
        })
        .take(limit)
        .collect()
}

fn agent_recent_traces(agent_id: &str, traces: &[AgentDecisionTrace], limit: usize) -> Vec<String> {
    traces
        .iter()
        .rev()
        .filter(|trace| trace.agent_id == agent_id)
        .flat_map(|trace| {
            let mut lines = Vec::new();
            lines.push(format!("- t{} decision {:?}", trace.time, trace.decision));
            if let Some(input) = trace.llm_input.as_ref() {
                lines.push(format!("  input: {}", truncate_text(input, 240)));
            }
            if let Some(output) = trace.llm_output.as_ref() {
                lines.push(format!("  output: {}", truncate_text(output, 240)));
            }
            if let Some(err) = trace.llm_error.as_ref() {
                lines.push(format!("  llm_error: {}", truncate_text(err, 160)));
            }
            if let Some(parse_error) = trace.parse_error.as_ref() {
                lines.push(format!(
                    "  parse_error: {}",
                    truncate_text(parse_error, 160)
                ));
            }
            if let Some(diagnostics) = trace.llm_diagnostics.as_ref() {
                lines.push(format!(
                    "  model: {}",
                    diagnostics.model.as_deref().unwrap_or("-")
                ));
                lines.push(format!(
                    "  latency_ms: {}",
                    diagnostics
                        .latency_ms
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string())
                ));
                lines.push(format!(
                    "  tokens: prompt={} completion={} total={}",
                    diagnostics
                        .prompt_tokens
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    diagnostics
                        .completion_tokens
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    diagnostics
                        .total_tokens
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string())
                ));
                lines.push(format!("  retries: {}", diagnostics.retry_count));
            }
            lines
        })
        .take(limit * 5)
        .collect()
}

fn truncate_text(text: &str, max_len: usize) -> String {
    let normalized = text.replace('\n', "\\n");
    if normalized.chars().count() <= max_len {
        return normalized;
    }
    normalized.chars().take(max_len).collect::<String>() + "..."
}

fn latest_agent_activity(agent_id: &str, events: &[WorldEvent]) -> Option<String> {
    for event in events.iter().rev() {
        if let Some(activity) = event_activity_for_agent(event, agent_id) {
            return Some(format!("t{} {}", event.time, activity));
        }
    }
    None
}

fn event_activity_for_agent(event: &WorldEvent, agent_id: &str) -> Option<String> {
    match &event.kind {
        WorldEventKind::AgentRegistered {
            agent_id: id,
            location_id,
            ..
        } if id == agent_id => Some(format!("register at {location_id}")),
        WorldEventKind::AgentMoved {
            agent_id: id,
            to,
            electricity_cost,
            ..
        } if id == agent_id => Some(format!("move -> {to} (cost {electricity_cost})")),
        WorldEventKind::RadiationHarvested {
            agent_id: id,
            amount,
            location_id,
            ..
        } if id == agent_id => Some(format!("harvest +{amount} at {location_id}")),
        WorldEventKind::ResourceTransferred {
            from,
            to,
            kind,
            amount,
        } => {
            let from_agent = owner_matches_agent(from, agent_id);
            let to_agent = owner_matches_agent(to, agent_id);
            match (from_agent, to_agent) {
                (true, true) => Some(format!("transfer {:?} {} (self)", kind, amount)),
                (true, false) => Some(format!("transfer out {:?} {}", kind, amount)),
                (false, true) => Some(format!("transfer in {:?} {}", kind, amount)),
                _ => None,
            }
        }
        WorldEventKind::CompoundRefined {
            owner,
            compound_mass_g,
            hardware_output,
            ..
        } if owner_matches_agent(owner, agent_id) => Some(format!(
            "refine {}g -> hw {}",
            compound_mass_g, hardware_output
        )),
        WorldEventKind::Power(power_event) => match power_event {
            PowerEvent::PowerConsumed {
                agent_id: id,
                amount,
                ..
            } if id == agent_id => Some(format!("power -{amount}")),
            PowerEvent::PowerStateChanged {
                agent_id: id, to, ..
            } if id == agent_id => Some(format!("power state -> {:?}", to)),
            PowerEvent::PowerCharged {
                agent_id: id,
                amount,
                ..
            } if id == agent_id => Some(format!("power +{amount}")),
            PowerEvent::PowerTransferred {
                from,
                to,
                amount,
                loss,
                ..
            } => {
                let from_agent = owner_matches_agent(from, agent_id);
                let to_agent = owner_matches_agent(to, agent_id);
                match (from_agent, to_agent) {
                    (true, true) => Some(format!("trade power {} (loss {})", amount, loss)),
                    (true, false) => Some(format!("sell power {} (loss {})", amount, loss)),
                    (false, true) => Some(format!("buy power {} (loss {})", amount, loss)),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

fn event_activity_for_location(event: &WorldEvent, location_id: &str) -> Option<String> {
    match &event.kind {
        WorldEventKind::LocationRegistered {
            location_id: id,
            name,
            ..
        } if id == location_id => Some(format!("register {name}")),
        WorldEventKind::AgentRegistered {
            agent_id,
            location_id: id,
            ..
        } if id == location_id => Some(format!("agent {agent_id} spawn")),
        WorldEventKind::AgentMoved {
            agent_id, from, to, ..
        } if from == location_id => Some(format!("agent {agent_id} moved out -> {to}")),
        WorldEventKind::AgentMoved {
            agent_id, from, to, ..
        } if to == location_id => Some(format!("agent {agent_id} moved in <- {from}")),
        WorldEventKind::RadiationHarvested {
            agent_id,
            location_id: id,
            amount,
            ..
        } if id == location_id => Some(format!("agent {agent_id} harvest +{amount}")),
        WorldEventKind::ResourceTransferred {
            from,
            to,
            kind,
            amount,
        } => {
            let from_location = owner_matches_location(from, location_id);
            let to_location = owner_matches_location(to, location_id);
            match (from_location, to_location) {
                (true, true) => Some(format!("transfer {:?} {} (self)", kind, amount)),
                (true, false) => Some(format!("transfer out {:?} {}", kind, amount)),
                (false, true) => Some(format!("transfer in {:?} {}", kind, amount)),
                _ => None,
            }
        }
        WorldEventKind::Power(PowerEvent::PowerGenerated {
            location_id: id,
            amount,
            plant_id,
        }) if id == location_id => Some(format!("plant {plant_id} generated {amount}")),
        WorldEventKind::Power(PowerEvent::PowerStored {
            location_id: id,
            stored,
            storage_id,
            ..
        }) if id == location_id => Some(format!("storage {storage_id} stored {stored}")),
        WorldEventKind::Power(PowerEvent::PowerDischarged {
            location_id: id,
            output,
            storage_id,
            ..
        }) if id == location_id => Some(format!("storage {storage_id} discharged {output}")),
        _ => None,
    }
}

fn event_activity_for_chunk(event: &WorldEvent, coord: ChunkCoord) -> Option<String> {
    match &event.kind {
        WorldEventKind::ChunkGenerated {
            coord: event_coord,
            fragment_count,
            block_count,
            cause,
            ..
        } if *event_coord == coord => Some(format!(
            "generated fragments={} blocks={} cause={:?}",
            fragment_count, block_count, cause
        )),
        _ => None,
    }
}

fn event_activity_for_module_visual(event: &WorldEvent, entity_id: &str) -> Option<String> {
    match &event.kind {
        WorldEventKind::ModuleVisualEntityUpserted { entity } if entity.entity_id == entity_id => {
            Some(format!(
                "upsert module={} kind={} anchor={}",
                entity.module_id,
                entity.kind,
                module_visual_anchor_label(&entity.anchor)
            ))
        }
        WorldEventKind::ModuleVisualEntityRemoved { entity_id: id } if id == entity_id => {
            Some("removed".to_string())
        }
        _ => None,
    }
}

fn event_activity_for_owner(event: &WorldEvent, owner: &ResourceOwner) -> Option<String> {
    match &event.kind {
        WorldEventKind::ResourceTransferred {
            from,
            to,
            kind,
            amount,
        } if from == owner && to == owner => Some(format!("transfer {:?} {} (self)", kind, amount)),
        WorldEventKind::ResourceTransferred {
            from,
            to: _,
            kind,
            amount,
        } if from == owner => Some(format!("transfer out {:?} {}", kind, amount)),
        WorldEventKind::ResourceTransferred {
            from: _,
            to,
            kind,
            amount,
        } if to == owner => Some(format!("transfer in {:?} {}", kind, amount)),
        WorldEventKind::CompoundRefined {
            owner: refined_owner,
            compound_mass_g,
            hardware_output,
            ..
        } if refined_owner == owner => Some(format!(
            "refine {}g -> hw {}",
            compound_mass_g, hardware_output
        )),
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from,
            to,
            amount,
            loss,
            ..
        }) if from == owner && to == owner => {
            Some(format!("trade power {} (loss {})", amount, loss))
        }
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from,
            to: _,
            amount,
            loss,
            ..
        }) if from == owner => Some(format!("sell power {} (loss {})", amount, loss)),
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from: _,
            to,
            amount,
            loss,
            ..
        }) if to == owner => Some(format!("buy power {} (loss {})", amount, loss)),
        _ => None,
    }
}

fn event_activity_for_power_plant(event: &WorldEvent, facility_id: &str) -> Option<String> {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerPlantRegistered { plant })
            if plant.id == facility_id =>
        {
            Some(format!("register at {}", plant.location_id))
        }
        WorldEventKind::Power(PowerEvent::PowerGenerated {
            plant_id,
            amount,
            location_id,
        }) if plant_id == facility_id => Some(format!("generated {} at {}", amount, location_id)),
        _ => None,
    }
}

fn event_activity_for_power_storage(event: &WorldEvent, storage_id: &str) -> Option<String> {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerStorageRegistered { storage })
            if storage.id == storage_id =>
        {
            Some(format!("register at {}", storage.location_id))
        }
        WorldEventKind::Power(PowerEvent::PowerStored {
            storage_id: id,
            input,
            stored,
            ..
        }) if id == storage_id => Some(format!("stored {} (input {})", stored, input)),
        WorldEventKind::Power(PowerEvent::PowerDischarged {
            storage_id: id,
            output,
            drawn,
            ..
        }) if id == storage_id => Some(format!("discharged {} (drawn {})", output, drawn)),
        _ => None,
    }
}

fn owner_matches_agent(owner: &ResourceOwner, agent_id: &str) -> bool {
    matches!(owner, ResourceOwner::Agent { agent_id: id } if id == agent_id)
}

fn owner_matches_location(owner: &ResourceOwner, location_id: &str) -> bool {
    matches!(owner, ResourceOwner::Location { location_id: id } if id == location_id)
}

#[cfg(test)]
#[path = "ui_text_tests.rs"]
mod tests;
