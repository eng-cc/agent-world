use agent_world::viewer::{ViewerControl, ViewerRequest};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::{ControlButton, ViewerClient, ViewerState};

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
    mut text_query: Query<&mut Text, With<TimelineStatusText>>,
    mut fill_query: Query<&mut Node, With<TimelineBarFill>>,
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

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!(
            "Timeline: now={} target={} max={} mode={}",
            current_tick, timeline.target_tick, axis_max, mode_label
        );
    }

    let progress = if axis_max == 0 {
        0.0
    } else {
        ((timeline.target_tick as f32) / (axis_max as f32) * 100.0).clamp(0.0, 100.0)
    };

    for mut fill in &mut fill_query {
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
