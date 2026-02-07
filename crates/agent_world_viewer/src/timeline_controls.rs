use agent_world::simulator::{PowerEvent, WorldEvent, WorldEventKind};
use agent_world::viewer::{ViewerControl, ViewerRequest};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::button_feedback::{mark_step_loading_on_control, StepControlLoadingState};
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

#[derive(Resource, Clone, Copy)]
pub(super) struct TimelineMarkFilterState {
    pub show_error: bool,
    pub show_llm: bool,
    pub show_peak: bool,
}

impl Default for TimelineMarkFilterState {
    fn default() -> Self {
        Self {
            show_error: true,
            show_llm: true,
            show_peak: true,
        }
    }
}

impl TimelineMarkFilterState {
    fn is_enabled(&self, kind: TimelineMarkKind) -> bool {
        match kind {
            TimelineMarkKind::Error => self.show_error,
            TimelineMarkKind::Llm => self.show_llm,
            TimelineMarkKind::Peak => self.show_peak,
        }
    }

    fn toggle(&mut self, kind: TimelineMarkKind) {
        match kind {
            TimelineMarkKind::Error => self.show_error = !self.show_error,
            TimelineMarkKind::Llm => self.show_llm = !self.show_llm,
            TimelineMarkKind::Peak => self.show_peak = !self.show_peak,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimelineMarkKind {
    Error,
    Llm,
    Peak,
}

#[derive(Component)]
pub(super) struct TimelineMarkJumpButton {
    kind: TimelineMarkKind,
}

#[derive(Component)]
pub(super) struct TimelineMarkFilterButton {
    kind: TimelineMarkKind,
}

#[derive(Component)]
pub(super) struct TimelineMarkFilterLabel {
    kind: TimelineMarkKind,
}

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
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.13, 0.14, 0.18)),
            BorderColor::all(Color::srgb(0.24, 0.26, 0.3)),
        ))
        .with_children(|timeline| {
            timeline.spawn((
                Text::new("Timeline: now=0 target=0 max=0"),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
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
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.8, 0.9)),
                TimelineInsightsText,
            ));

            timeline
                .spawn(Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(24.0),
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|filters| {
                    spawn_mark_filter_button(filters, &font, TimelineMarkKind::Error);
                    spawn_mark_filter_button(filters, &font, TimelineMarkKind::Llm);
                    spawn_mark_filter_button(filters, &font, TimelineMarkKind::Peak);
                });

            timeline
                .spawn(Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(24.0),
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|marks| {
                    spawn_mark_jump_button(
                        marks,
                        &font,
                        "Jump Err",
                        TimelineMarkKind::Error,
                        Color::srgb(0.42, 0.2, 0.2),
                    );
                    spawn_mark_jump_button(
                        marks,
                        &font,
                        "Jump LLM",
                        TimelineMarkKind::Llm,
                        Color::srgb(0.2, 0.32, 0.42),
                    );
                    spawn_mark_jump_button(
                        marks,
                        &font,
                        "Jump Peak",
                        TimelineMarkKind::Peak,
                        Color::srgb(0.32, 0.28, 0.16),
                    );
                });

            timeline
                .spawn(Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(28.0),
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
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
                                min_width: Val::Px(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.28, 0.22)),
                            TimelineSeekSubmitButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Seek"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
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
                min_width: Val::Px(44.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                height: Val::Px(24.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.26)),
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

fn spawn_mark_filter_button(
    buttons: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    kind: TimelineMarkKind,
) {
    let enabled = true;
    buttons
        .spawn((
            Button,
            Node {
                min_width: Val::Px(78.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                height: Val::Px(22.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(mark_filter_background(kind, enabled)),
            TimelineMarkFilterButton { kind },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(mark_filter_label(kind, enabled)),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimelineMarkFilterLabel { kind },
            ));
        });
}

fn spawn_mark_jump_button(
    buttons: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    kind: TimelineMarkKind,
    background: Color,
) {
    buttons
        .spawn((
            Button,
            Node {
                min_width: Val::Px(88.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                height: Val::Px(22.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(background),
            TimelineMarkJumpButton { kind },
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

pub(super) fn handle_timeline_mark_filter_buttons(
    mut filters: ResMut<TimelineMarkFilterState>,
    mut interactions: Query<
        (&Interaction, &TimelineMarkFilterButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            filters.toggle(button.kind);
        }
    }
}

pub(super) fn update_timeline_mark_filter_ui(
    filters: Res<TimelineMarkFilterState>,
    mut button_query: Query<(&TimelineMarkFilterButton, &mut BackgroundColor)>,
    mut label_query: Query<(&TimelineMarkFilterLabel, &mut Text)>,
) {
    if !filters.is_changed() {
        return;
    }

    for (button, mut background) in &mut button_query {
        let enabled = filters.is_enabled(button.kind);
        background.0 = mark_filter_background(button.kind, enabled);
    }

    for (label, mut text) in &mut label_query {
        let enabled = filters.is_enabled(label.kind);
        text.0 = mark_filter_label(label.kind, enabled);
    }
}

pub(super) fn handle_timeline_mark_jump_buttons(
    state: Res<ViewerState>,
    mut timeline: ResMut<TimelineUiState>,
    mark_filters: Option<Res<TimelineMarkFilterState>>,
    mut interactions: Query<
        (&Interaction, &TimelineMarkJumpButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let axis_max = timeline_axis_max(&timeline, current_tick_from_state(&state));
    let insights = apply_mark_filters(
        build_timeline_key_insights(&state.events, &state.decision_traces, axis_max),
        mark_filters.as_ref().map(|filters| filters.as_ref()),
    );

    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let ticks = match button.kind {
            TimelineMarkKind::Error => insights.error_ticks.as_slice(),
            TimelineMarkKind::Llm => insights.llm_ticks.as_slice(),
            TimelineMarkKind::Peak => insights.resource_peak_ticks.as_slice(),
        };

        if let Some(next_tick) = select_next_mark_tick(ticks, timeline.target_tick) {
            timeline.target_tick = next_tick;
            timeline.manual_override = true;
            timeline.drag_active = false;
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
    mark_filters: Option<Res<TimelineMarkFilterState>>,
    mut queries: ParamSet<(
        Query<&mut Text, With<TimelineStatusText>>,
        Query<&mut Text, With<TimelineInsightsText>>,
        Query<&mut Node, With<TimelineBarFill>>,
    )>,
) {
    let filter_changed = mark_filters
        .as_ref()
        .map(|filters| filters.is_changed())
        .unwrap_or(false);
    if !state.is_changed() && !timeline.is_changed() && !filter_changed {
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
        let filters = mark_filters.as_ref().map(|filters| filters.as_ref());
        let key = apply_mark_filters(
            build_timeline_key_insights(&state.events, &state.decision_traces, axis_max),
            filters,
        );
        let filter_state = filters.copied().unwrap_or_default();
        text.0 = format!(
            "Marks: err={} llm={} peak={}\nTicks: E[{}] L[{}] P[{}]\nFilter: err={} llm={} peak={}\nDensity: {}",
            key.error_ticks.len(),
            key.llm_ticks.len(),
            key.resource_peak_ticks.len(),
            format_tick_list(&key.error_ticks, MAX_TICK_LABELS),
            format_tick_list(&key.llm_ticks, MAX_TICK_LABELS),
            format_tick_list(&key.resource_peak_ticks, MAX_TICK_LABELS),
            enabled_label(filter_state.show_error),
            enabled_label(filter_state.show_llm),
            enabled_label(filter_state.show_peak),
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

fn select_next_mark_tick(ticks: &[u64], current_target: u64) -> Option<u64> {
    ticks
        .iter()
        .copied()
        .find(|tick| *tick > current_target)
        .or_else(|| ticks.first().copied())
}

fn apply_mark_filters(
    mut insights: TimelineKeyInsights,
    filters: Option<&TimelineMarkFilterState>,
) -> TimelineKeyInsights {
    let filters = filters.copied().unwrap_or_default();
    if !filters.show_error {
        insights.error_ticks.clear();
    }
    if !filters.show_llm {
        insights.llm_ticks.clear();
    }
    if !filters.show_peak {
        insights.resource_peak_ticks.clear();
    }
    insights
}

fn mark_filter_background(kind: TimelineMarkKind, enabled: bool) -> Color {
    if !enabled {
        return Color::srgb(0.16, 0.16, 0.18);
    }
    match kind {
        TimelineMarkKind::Error => Color::srgb(0.52, 0.22, 0.22),
        TimelineMarkKind::Llm => Color::srgb(0.22, 0.4, 0.52),
        TimelineMarkKind::Peak => Color::srgb(0.42, 0.36, 0.18),
    }
}

fn mark_filter_label(kind: TimelineMarkKind, enabled: bool) -> String {
    let prefix = match kind {
        TimelineMarkKind::Error => "Err",
        TimelineMarkKind::Llm => "LLM",
        TimelineMarkKind::Peak => "Peak",
    };
    format!("{}:{}", prefix, if enabled { "ON" } else { "OFF" })
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled {
        "on"
    } else {
        "off"
    }
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
    state: Res<ViewerState>,
    mut loading: ResMut<StepControlLoadingState>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if matches!(button.control, ViewerControl::Step { .. }) && loading.pending {
            continue;
        }

        mark_step_loading_on_control(&button.control, &state, &mut loading);
        let _ = client.tx.send(ViewerRequest::Control {
            mode: button.control.clone(),
        });
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
                llm_diagnostics: None,
            },
            AgentDecisionTrace {
                agent_id: "agent-2".to_string(),
                time: 9,
                decision: AgentDecision::Wait,
                llm_input: None,
                llm_output: None,
                llm_error: None,
                parse_error: None,
                llm_diagnostics: None,
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

    #[test]
    fn select_next_mark_tick_cycles_forward_then_wraps() {
        let ticks = vec![3, 8, 12];
        assert_eq!(select_next_mark_tick(&ticks, 0), Some(3));
        assert_eq!(select_next_mark_tick(&ticks, 3), Some(8));
        assert_eq!(select_next_mark_tick(&ticks, 11), Some(12));
        assert_eq!(select_next_mark_tick(&ticks, 99), Some(3));
        assert_eq!(select_next_mark_tick(&[], 10), None);
    }

    #[test]
    fn mark_filter_button_toggles_state() {
        let mut app = App::new();
        app.add_systems(Update, handle_timeline_mark_filter_buttons);

        app.world_mut()
            .insert_resource(TimelineMarkFilterState::default());
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            TimelineMarkFilterButton {
                kind: TimelineMarkKind::Error,
            },
        ));

        app.update();

        let filters = app.world().resource::<TimelineMarkFilterState>();
        assert!(!filters.show_error);
        assert!(filters.show_llm);
        assert!(filters.show_peak);
    }

    #[test]
    fn mark_jump_respects_disabled_filter() {
        let mut app = App::new();
        app.add_systems(Update, handle_timeline_mark_jump_buttons);

        let events = vec![WorldEvent {
            id: 1,
            time: 5,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 1 },
            },
        }];

        app.world_mut().insert_resource(ViewerState {
            status: crate::ConnectionStatus::Connected,
            snapshot: None,
            events,
            decision_traces: Vec::new(),
            metrics: None,
        });
        app.world_mut().insert_resource(TimelineUiState {
            target_tick: 0,
            max_tick_seen: 10,
            manual_override: false,
            drag_active: false,
        });
        app.world_mut().insert_resource(TimelineMarkFilterState {
            show_error: false,
            show_llm: true,
            show_peak: true,
        });

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            TimelineMarkJumpButton {
                kind: TimelineMarkKind::Error,
            },
        ));

        app.update();

        let timeline = app.world().resource::<TimelineUiState>();
        assert_eq!(timeline.target_tick, 0);
        assert!(!timeline.manual_override);
    }

    #[test]
    fn mark_jump_button_updates_timeline_target() {
        let mut app = App::new();
        app.add_systems(Update, handle_timeline_mark_jump_buttons);

        let events = vec![WorldEvent {
            id: 1,
            time: 5,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 1 },
            },
        }];

        app.world_mut().insert_resource(ViewerState {
            status: crate::ConnectionStatus::Connected,
            snapshot: None,
            events,
            decision_traces: Vec::new(),
            metrics: None,
        });
        app.world_mut().insert_resource(TimelineUiState {
            target_tick: 0,
            max_tick_seen: 10,
            manual_override: false,
            drag_active: false,
        });

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            TimelineMarkJumpButton {
                kind: TimelineMarkKind::Error,
            },
        ));

        app.update();

        let timeline = app.world().resource::<TimelineUiState>();
        assert_eq!(timeline.target_tick, 5);
        assert!(timeline.manual_override);
    }
}
