use agent_world::simulator::{PowerEvent, WorldEvent, WorldEventKind};
use agent_world::viewer::{ViewerControl, ViewerRequest};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::{ControlButton, ViewerClient, ViewerState};

const DENSITY_BINS: usize = 16;
const MAX_TICK_LABELS: usize = 4;
const MAX_PEAK_TICKS: usize = 3;

#[derive(Resource, Default)]
pub(super) struct TimelineUiState {
    pub target_tick: u64,
    pub max_tick_seen: u64,
    pub manual_override: bool,
    pub drag_active: bool,
}

#[derive(Component)]
pub(super) struct TimelineAdjustButton {
    pub delta: i64,
}

#[derive(Component)]
pub(super) struct TimelineSeekSubmitButton;

#[derive(Component)]
pub(super) struct TimelineBar;

#[derive(Component)]
pub(super) struct TimelineBarFill;

#[derive(Component)]
pub(super) struct TimelineStatusText;

#[derive(Component)]
pub(super) struct TimelineInsightsText;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimelineKeyInsights {
    error_ticks: Vec<u64>,
    llm_ticks: Vec<u64>,
    resource_peak_ticks: Vec<u64>,
    density_sparkline: String,
}

pub(super) fn spawn_timeline_controls(parent: &mut ChildSpawnerCommands, font: Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.14, 0.16)),
        ))
        .with_children(|timeline| {
            timeline.spawn((
                Text::new("Timeline: now=0 target=0 max=0"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.9, 0.95)),
                TimelineStatusText,
            ));

            timeline.spawn((
                Text::new(
                    "Marks: err=0 llm=0 peak=0\nTicks: E[-] L[-] P[-]\nDensity: ················",
                ),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.8, 0.9)),
                TimelineInsightsText,
            ));

            timeline
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    column_gap: Val::Px(6.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|buttons| {
                    spawn_adjust_button(buttons, &font, "-100", -100);
                    spawn_adjust_button(buttons, &font, "-10", -10);
                    spawn_adjust_button(buttons, &font, "-1", -1);
                    spawn_adjust_button(buttons, &font, "+1", 1);
                    spawn_adjust_button(buttons, &font, "+10", 10);
                    spawn_adjust_button(buttons, &font, "+100", 100);

                    buttons
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                height: Val::Px(24.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.28, 0.22)),
                            TimelineSeekSubmitButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Seek Target"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            timeline
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.24)),
                    RelativeCursorPosition::default(),
                    TimelineBar,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.45, 0.62, 0.95)),
                        TimelineBarFill,
                    ));
                });
        });
}

fn spawn_adjust_button(
    buttons: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    delta: i64,
) {
    buttons
        .spawn((
            Button,
            Node {
                padding: UiRect::horizontal(Val::Px(8.0)),
                height: Val::Px(24.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.24)),
            TimelineAdjustButton { delta },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub(super) fn sync_timeline_state_from_world(
    mut timeline: ResMut<TimelineUiState>,
    state: Res<ViewerState>,
) {
    if !state.is_changed() {
        return;
    }

    let current_tick = current_tick_from_state(&state);
    timeline.max_tick_seen = timeline.max_tick_seen.max(current_tick);

    if !timeline.manual_override && !timeline.drag_active {
        timeline.target_tick = current_tick;
    }
}

pub(super) fn handle_timeline_adjust_buttons(
    mut interactions: Query<
        (&Interaction, &TimelineAdjustButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut timeline: ResMut<TimelineUiState>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        timeline.manual_override = true;
        if button.delta < 0 {
            timeline.target_tick = timeline.target_tick.saturating_sub((-button.delta) as u64);
        } else {
            timeline.target_tick = timeline.target_tick.saturating_add(button.delta as u64);
        }
    }
}

pub(super) fn handle_timeline_seek_submit(
    mut interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<TimelineSeekSubmitButton>,
        ),
    >,
    client: Res<ViewerClient>,
    mut timeline: ResMut<TimelineUiState>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            let _ = client.tx.send(ViewerRequest::Control {
                mode: ViewerControl::Seek {
                    tick: timeline.target_tick,
                },
            });
            timeline.manual_override = false;
            timeline.drag_active = false;
        }
    }
}

pub(super) fn handle_timeline_bar_drag(
    state: Res<ViewerState>,
    mut timeline: ResMut<TimelineUiState>,
    interactions: Query<(&Interaction, &RelativeCursorPosition), With<TimelineBar>>,
) {
    let current_tick = current_tick_from_state(&state);
    for (interaction, relative) in &interactions {
        if *interaction == Interaction::Pressed {
            timeline.drag_active = true;
            timeline.manual_override = true;
            if let Some(cursor) = relative.normalized {
                let axis_max = timeline_axis_max(&timeline, current_tick);
                timeline.target_tick = normalized_x_to_tick(cursor.x, axis_max);
            }
        } else if timeline.drag_active {
            timeline.drag_active = false;
        }
    }
}

pub(super) fn update_timeline_ui(
    state: Res<ViewerState>,
    timeline: Res<TimelineUiState>,
    mut queries: ParamSet<(
        Query<&mut Text, With<TimelineStatusText>>,
        Query<&mut Text, With<TimelineInsightsText>>,
        Query<&mut Node, With<TimelineBarFill>>,
    )>,
) {
    if !state.is_changed() && !timeline.is_changed() {
        return;
    }

    let current_tick = current_tick_from_state(&state);
    let axis_max = timeline_axis_max(&timeline, current_tick);
    let mode_label = if timeline.drag_active {
        "dragging"
    } else if timeline.manual_override {
        "manual"
    } else {
        "follow"
    };

    if let Ok(mut text) = queries.p0().single_mut() {
        text.0 = format!(
            "Timeline: now={} target={} max={} mode={}",
            current_tick, timeline.target_tick, axis_max, mode_label
        );
    }

    if let Ok(mut text) = queries.p1().single_mut() {
        let key = build_timeline_key_insights(&state.events, &state.decision_traces, axis_max);
        text.0 = format!(
            "Marks: err={} llm={} peak={}\nTicks: E[{}] L[{}] P[{}]\nDensity: {}",
            key.error_ticks.len(),
            key.llm_ticks.len(),
            key.resource_peak_ticks.len(),
            format_tick_list(&key.error_ticks, MAX_TICK_LABELS),
            format_tick_list(&key.llm_ticks, MAX_TICK_LABELS),
            format_tick_list(&key.resource_peak_ticks, MAX_TICK_LABELS),
            key.density_sparkline,
        );
    }

    let progress = if axis_max == 0 {
        0.0
    } else {
        ((timeline.target_tick as f32) / (axis_max as f32) * 100.0).clamp(0.0, 100.0)
    };

    for mut fill in &mut queries.p2() {
        fill.width = Val::Percent(progress);
    }
}

pub(super) fn normalized_x_to_tick(normalized_x: f32, axis_max: u64) -> u64 {
    if axis_max == 0 {
        return 0;
    }
    let ratio = (normalized_x + 0.5).clamp(0.0, 1.0);
    (ratio * axis_max as f32).round() as u64
}

fn current_tick_from_state(state: &ViewerState) -> u64 {
    state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0)
}

fn timeline_axis_max(timeline: &TimelineUiState, current_tick: u64) -> u64 {
    timeline
        .max_tick_seen
        .max(current_tick)
        .max(timeline.target_tick)
}

fn build_timeline_key_insights(
    events: &[WorldEvent],
    decision_traces: &[agent_world::simulator::AgentDecisionTrace],
    axis_max: u64,
) -> TimelineKeyInsights {
    let error_ticks = collect_error_ticks(events);
    let llm_ticks = collect_llm_ticks(decision_traces);
    let resource_peak_ticks = collect_resource_peak_ticks(events, MAX_PEAK_TICKS);
    let density_sparkline = event_density_sparkline(events, axis_max, DENSITY_BINS);
    TimelineKeyInsights {
        error_ticks,
        llm_ticks,
        resource_peak_ticks,
        density_sparkline,
    }
}

fn collect_error_ticks(events: &[WorldEvent]) -> Vec<u64> {
    let mut ticks = Vec::new();
    for event in events {
        if matches!(event.kind, WorldEventKind::ActionRejected { .. }) {
            ticks.push(event.time);
        }
    }
    dedup_sorted_ticks(ticks)
}

fn collect_llm_ticks(decision_traces: &[agent_world::simulator::AgentDecisionTrace]) -> Vec<u64> {
    let ticks: Vec<u64> = decision_traces.iter().map(|trace| trace.time).collect();
    dedup_sorted_ticks(ticks)
}

fn collect_resource_peak_ticks(events: &[WorldEvent], max_ticks: usize) -> Vec<u64> {
    let mut weighted_ticks: Vec<(i64, u64)> = events
        .iter()
        .filter_map(|event| event_resource_weight(event).map(|weight| (weight, event.time)))
        .collect();
    weighted_ticks.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));

    let mut selected = Vec::new();
    for (_, tick) in weighted_ticks {
        if !selected.contains(&tick) {
            selected.push(tick);
            if selected.len() >= max_ticks {
                break;
            }
        }
    }
    selected.sort_unstable();
    selected
}

fn event_resource_weight(event: &WorldEvent) -> Option<i64> {
    match &event.kind {
        WorldEventKind::ResourceTransferred { amount, .. } => Some(amount.abs()),
        WorldEventKind::RadiationHarvested { amount, .. } => Some(amount.abs()),
        WorldEventKind::CompoundRefined {
            electricity_cost,
            hardware_output,
            ..
        } => Some(electricity_cost.abs().saturating_add(hardware_output.abs())),
        WorldEventKind::Power(power_event) => power_event_weight(power_event),
        _ => None,
    }
}

fn power_event_weight(power_event: &PowerEvent) -> Option<i64> {
    match power_event {
        PowerEvent::PowerGenerated { amount, .. } => Some(amount.abs()),
        PowerEvent::PowerStored { stored, .. } => Some(stored.abs()),
        PowerEvent::PowerDischarged { output, .. } => Some(output.abs()),
        PowerEvent::PowerConsumed { amount, .. } => Some(amount.abs()),
        PowerEvent::PowerTransferred { amount, loss, .. } => {
            Some(amount.abs().saturating_add(loss.abs()))
        }
        PowerEvent::PowerCharged { amount, .. } => Some(amount.abs()),
        _ => None,
    }
}

fn event_density_sparkline(events: &[WorldEvent], axis_max: u64, bins: usize) -> String {
    if bins == 0 {
        return String::new();
    }

    let mut counts = vec![0_u32; bins];
    for event in events {
        let idx = tick_to_bin(event.time.min(axis_max), axis_max, bins);
        counts[idx] = counts[idx].saturating_add(1);
    }

    let max_count = counts.iter().copied().max().unwrap_or(0);
    if max_count == 0 {
        return "·".repeat(bins);
    }

    const LEVELS: [char; 9] = ['·', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    counts
        .iter()
        .map(|count| {
            if *count == 0 {
                return LEVELS[0];
            }
            let scaled = ((*count as f32 / max_count as f32) * 8.0).ceil() as usize;
            LEVELS[scaled.clamp(1, 8)]
        })
        .collect()
}

fn tick_to_bin(tick: u64, axis_max: u64, bins: usize) -> usize {
    if axis_max == 0 || bins <= 1 {
        return 0;
    }
    let ratio = (tick as f32 / axis_max as f32).clamp(0.0, 1.0);
    (ratio * (bins.saturating_sub(1)) as f32).round() as usize
}

fn dedup_sorted_ticks(mut ticks: Vec<u64>) -> Vec<u64> {
    ticks.sort_unstable();
    ticks.dedup();
    ticks
}

fn format_tick_list(ticks: &[u64], max_items: usize) -> String {
    if ticks.is_empty() {
        return "-".to_string();
    }
    let shown: Vec<String> = ticks
        .iter()
        .take(max_items)
        .map(|tick| tick.to_string())
        .collect();
    if ticks.len() > max_items {
        format!("{}+{}", shown.join(","), ticks.len() - max_items)
    } else {
        shown.join(",")
    }
}

pub(super) fn handle_control_buttons(
    mut interactions: Query<(&Interaction, &ControlButton), (Changed<Interaction>, With<Button>)>,
    client: Res<ViewerClient>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            let _ = client.tx.send(ViewerRequest::Control {
                mode: button.control.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::simulator::{
        AgentDecision, AgentDecisionTrace, ConsumeReason, PowerEvent, RejectReason,
    };

    #[test]
    fn key_insights_collects_error_llm_and_peaks() {
        let events = vec![
            WorldEvent {
                id: 1,
                time: 2,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InvalidAmount { amount: 1 },
                },
            },
            WorldEvent {
                id: 2,
                time: 6,
                kind: WorldEventKind::Power(PowerEvent::PowerGenerated {
                    plant_id: "plant-1".to_string(),
                    location_id: "loc-1".to_string(),
                    amount: 120,
                }),
            },
            WorldEvent {
                id: 3,
                time: 8,
                kind: WorldEventKind::ResourceTransferred {
                    from: agent_world::simulator::ResourceOwner::Location {
                        location_id: "loc-1".to_string(),
                    },
                    to: agent_world::simulator::ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    kind: agent_world::simulator::ResourceKind::Electricity,
                    amount: 300,
                },
            },
            WorldEvent {
                id: 4,
                time: 1,
                kind: WorldEventKind::Power(PowerEvent::PowerConsumed {
                    agent_id: "agent-1".to_string(),
                    amount: 7,
                    reason: ConsumeReason::Decision,
                    remaining: 10,
                }),
            },
        ];

        let traces = vec![
            AgentDecisionTrace {
                agent_id: "agent-1".to_string(),
                time: 4,
                decision: AgentDecision::Wait,
                llm_input: None,
                llm_output: None,
                llm_error: None,
                parse_error: None,
            },
            AgentDecisionTrace {
                agent_id: "agent-2".to_string(),
                time: 9,
                decision: AgentDecision::Wait,
                llm_input: None,
                llm_output: None,
                llm_error: None,
                parse_error: None,
            },
        ];

        let summary = build_timeline_key_insights(&events, &traces, 10);
        assert_eq!(summary.error_ticks, vec![2]);
        assert_eq!(summary.llm_ticks, vec![4, 9]);
        assert_eq!(summary.resource_peak_ticks, vec![1, 6, 8]);
        assert_eq!(summary.density_sparkline.chars().count(), DENSITY_BINS);
    }

    #[test]
    fn density_sparkline_reflects_event_distribution() {
        let events = vec![
            WorldEvent {
                id: 1,
                time: 0,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InvalidAmount { amount: 1 },
                },
            },
            WorldEvent {
                id: 2,
                time: 5,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InvalidAmount { amount: 1 },
                },
            },
            WorldEvent {
                id: 3,
                time: 5,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InvalidAmount { amount: 1 },
                },
            },
            WorldEvent {
                id: 4,
                time: 10,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InvalidAmount { amount: 1 },
                },
            },
        ];

        let sparkline = event_density_sparkline(&events, 10, 5);
        assert_eq!(sparkline.chars().count(), 5);
        assert!(sparkline.contains('█'));
        assert!(sparkline.contains('·'));
    }

    #[test]
    fn format_tick_list_compacts_output() {
        assert_eq!(format_tick_list(&[], 4), "-");
        assert_eq!(format_tick_list(&[1, 2, 3], 4), "1,2,3");
        assert_eq!(format_tick_list(&[1, 2, 3, 4, 5], 3), "1,2,3+2");
    }
}
