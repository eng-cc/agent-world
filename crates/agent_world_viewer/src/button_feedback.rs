use bevy::prelude::*;

use crate::i18n::{locale_or_default, step_button_label, UiI18n};

use super::{ControlButton, ViewerControl, ViewerState};

#[derive(Component, Clone, Copy)]
pub(super) struct StepButton;

#[derive(Component)]
pub(super) struct StepButtonLabel;

#[derive(Component, Clone, Copy)]
pub(super) struct ButtonVisualBase {
    color: Color,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StepControlLoadingState {
    pub pending: bool,
    pub baseline_tick: u64,
}

const STEP_LOADING_COLOR: Color = Color::srgb(0.5, 0.35, 0.1);

pub(super) fn attach_step_button_markers(
    mut commands: Commands,
    buttons: Query<
        (Entity, &ControlButton, Option<&Children>),
        (With<Button>, Without<StepButton>),
    >,
    texts: Query<(), With<Text>>,
) {
    for (entity, control, children) in &buttons {
        if !matches!(control.control, ViewerControl::Step { .. }) {
            continue;
        }

        commands.entity(entity).insert(StepButton);

        if let Some(children) = children {
            for child in children.iter() {
                if texts.get(child).is_ok() {
                    commands.entity(child).insert(StepButtonLabel);
                    break;
                }
            }
        }
    }
}

pub(super) fn init_button_visual_base(
    mut commands: Commands,
    buttons: Query<(Entity, &BackgroundColor), (With<Button>, Without<ButtonVisualBase>)>,
) {
    for (entity, background) in &buttons {
        commands.entity(entity).insert(ButtonVisualBase {
            color: background.0,
        });
    }
}

pub(super) fn update_button_hover_visuals(
    loading: Res<StepControlLoadingState>,
    mut query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonVisualBase,
            Option<&StepButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background, base, step_button) in &mut query {
        if step_button.is_some() && loading.pending {
            continue;
        }

        background.0 = match *interaction {
            Interaction::None => base.color,
            Interaction::Hovered => highlight_color(base.color, 0.18),
            Interaction::Pressed => highlight_color(base.color, -0.12),
        };
    }
}

pub(super) fn track_step_loading_state(
    mut loading: ResMut<StepControlLoadingState>,
    state: Res<ViewerState>,
) {
    if !loading.pending || !state.is_changed() {
        return;
    }

    if let Some(snapshot) = state.snapshot.as_ref() {
        if snapshot.time > loading.baseline_tick {
            loading.pending = false;
        }
    }

    if matches!(state.status, super::ConnectionStatus::Error(_)) {
        loading.pending = false;
    }
}

pub(super) fn update_step_button_loading_ui(
    loading: Res<StepControlLoadingState>,
    i18n: Option<Res<UiI18n>>,
    mut button_query: Query<(&mut BackgroundColor, &ButtonVisualBase), With<StepButton>>,
    mut label_query: Query<&mut Text, With<StepButtonLabel>>,
) {
    let locale_changed = i18n
        .as_ref()
        .map(|value| value.is_changed())
        .unwrap_or(false);
    if !loading.is_changed() && !locale_changed {
        return;
    }

    let locale = locale_or_default(i18n.as_deref());

    for (mut background, base) in &mut button_query {
        background.0 = if loading.pending {
            STEP_LOADING_COLOR
        } else {
            base.color
        };
    }

    for mut label in &mut label_query {
        label.0 = step_button_label(locale, loading.pending).to_string();
    }
}

pub(super) fn mark_step_loading_on_control(
    control: &ViewerControl,
    state: &ViewerState,
    loading: &mut StepControlLoadingState,
) {
    match control {
        ViewerControl::Step { .. } => {
            if loading.pending {
                return;
            }
            loading.pending = true;
            loading.baseline_tick = state
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.time)
                .unwrap_or(0);
        }
        ViewerControl::Play | ViewerControl::Pause | ViewerControl::Seek { .. } => {
            loading.pending = false;
        }
    }
}

fn highlight_color(base: Color, amount: f32) -> Color {
    let mut srgba = base.to_srgba();
    if amount >= 0.0 {
        srgba.red = srgba.red + (1.0 - srgba.red) * amount;
        srgba.green = srgba.green + (1.0 - srgba.green) * amount;
        srgba.blue = srgba.blue + (1.0 - srgba.blue) * amount;
    } else {
        let factor = 1.0 + amount;
        srgba.red *= factor;
        srgba.green *= factor;
        srgba.blue *= factor;
    }
    Color::srgba(srgba.red, srgba.green, srgba.blue, srgba.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_color_lightens_and_darkens() {
        let base = Color::srgb(0.2, 0.3, 0.4);
        let lighter = highlight_color(base, 0.2).to_srgba();
        let darker = highlight_color(base, -0.2).to_srgba();

        assert!(lighter.red > base.to_srgba().red);
        assert!(lighter.green > base.to_srgba().green);
        assert!(darker.blue < base.to_srgba().blue);
    }

    #[test]
    fn mark_step_loading_uses_snapshot_tick_baseline() {
        let state = ViewerState {
            status: super::super::ConnectionStatus::Connected,
            snapshot: Some(agent_world::simulator::WorldSnapshot {
                version: agent_world::simulator::SNAPSHOT_VERSION,
                chunk_generation_schema_version:
                    agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
                time: 11,
                config: agent_world::simulator::WorldConfig::default(),
                model: agent_world::simulator::WorldModel::default(),
                chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
                next_event_id: 1,
                next_action_id: 1,
                pending_actions: Vec::new(),
                journal_len: 0,
            }),
            events: Vec::new(),
            decision_traces: Vec::new(),
            metrics: None,
        };
        let mut loading = StepControlLoadingState::default();

        mark_step_loading_on_control(&ViewerControl::Step { count: 1 }, &state, &mut loading);

        assert!(loading.pending);
        assert_eq!(loading.baseline_tick, 11);

        mark_step_loading_on_control(&ViewerControl::Pause, &state, &mut loading);
        assert!(!loading.pending);
    }
}
