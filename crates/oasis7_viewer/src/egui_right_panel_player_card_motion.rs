use super::egui_right_panel_player_experience::PlayerGuideStep;

const PLAYER_CARD_ENTRY_SECS: f64 = 0.48;
const PLAYER_CARD_ENTRY_SLIDE_PX: f32 = 18.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PlayerGuideTransitionState {
    active_step: Option<PlayerGuideStep>,
    started_at_secs: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlayerCardTransitionSnapshot {
    pub(super) alpha: f32,
    pub(super) slide_px: f32,
    pub(super) pulse: f32,
}

pub(super) fn sync_player_guide_transition(
    transition: &mut PlayerGuideTransitionState,
    step: PlayerGuideStep,
    now_secs: f64,
) {
    if transition.active_step != Some(step) {
        transition.active_step = Some(step);
        transition.started_at_secs = now_secs;
    }
}

pub(super) fn build_player_card_transition_snapshot(
    transition: &PlayerGuideTransitionState,
    step: PlayerGuideStep,
    now_secs: f64,
    pulse_hz: f64,
) -> PlayerCardTransitionSnapshot {
    let progress = if transition.active_step == Some(step) {
        ((now_secs - transition.started_at_secs).max(0.0) / PLAYER_CARD_ENTRY_SECS).clamp(0.0, 1.0)
            as f32
    } else {
        1.0
    };
    let eased = 1.0 - (1.0 - progress).powi(3);
    let pulse = ((now_secs * pulse_hz).sin() * 0.5 + 0.5) as f32;
    PlayerCardTransitionSnapshot {
        alpha: (0.44 + 0.56 * eased).clamp(0.0, 1.0),
        slide_px: ((1.0 - eased) * PLAYER_CARD_ENTRY_SLIDE_PX).max(0.0),
        pulse,
    }
}
