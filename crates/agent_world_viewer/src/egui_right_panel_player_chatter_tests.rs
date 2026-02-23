use super::egui_right_panel_player_experience::{
    player_agent_chatter_cap, player_agent_chatter_ids, player_agent_chatter_last_seen_event_id,
    player_agent_chatter_len, player_agent_chatter_snapshot, sync_agent_chatter_bubbles,
};
use super::*;

#[test]
fn sync_agent_chatter_bubbles_skips_history_then_tracks_new_events_only() {
    let mut achievements = PlayerAchievementState::default();
    let mut state = sample_viewer_state(
        crate::ConnectionStatus::Connected,
        vec![sample_agent_moved_event(1, 1), sample_rejected_event(2, 2)],
    );
    let locale = crate::i18n::UiLocale::ZhCn;

    sync_agent_chatter_bubbles(&mut achievements, &state, 10.0, locale);
    assert_eq!(player_agent_chatter_len(&achievements), 0);
    assert_eq!(
        player_agent_chatter_last_seen_event_id(&achievements),
        Some(2)
    );

    state.events.push(sample_agent_moved_event(3, 3));
    sync_agent_chatter_bubbles(&mut achievements, &state, 11.0, locale);

    assert_eq!(
        player_agent_chatter_last_seen_event_id(&achievements),
        Some(3)
    );
    assert_eq!(player_agent_chatter_len(&achievements), 1);

    let snapshot = player_agent_chatter_snapshot(&achievements, 0)
        .expect("expected one chatter bubble after new agent event");
    assert_eq!(snapshot.0, 3);
    assert_eq!(snapshot.1, FeedbackTone::Positive);
    assert!(snapshot.2.contains("agent-3"));
    assert!(snapshot.3.contains("移动"));
}

#[test]
fn sync_agent_chatter_bubbles_clamps_queue_and_expires() {
    let mut achievements = PlayerAchievementState::default();
    let mut state = sample_viewer_state(
        crate::ConnectionStatus::Connected,
        vec![sample_agent_moved_event(1, 1)],
    );
    let locale = crate::i18n::UiLocale::EnUs;

    sync_agent_chatter_bubbles(&mut achievements, &state, 20.0, locale);
    assert_eq!(player_agent_chatter_len(&achievements), 0);
    assert_eq!(
        player_agent_chatter_last_seen_event_id(&achievements),
        Some(1)
    );

    let newest_id = player_agent_chatter_cap() as u64 + 3;
    for id in 2..=newest_id {
        state.events.push(sample_agent_moved_event(id, id));
        sync_agent_chatter_bubbles(&mut achievements, &state, 20.0 + id as f64, locale);
    }

    let ids = player_agent_chatter_ids(&achievements);
    let oldest_id = newest_id + 1 - player_agent_chatter_cap() as u64;
    let expected_ids: Vec<u64> = (oldest_id..=newest_id).collect();
    assert_eq!(ids, expected_ids);
    assert_eq!(
        player_agent_chatter_len(&achievements),
        player_agent_chatter_cap()
    );

    sync_agent_chatter_bubbles(&mut achievements, &state, 120.0, locale);
    assert_eq!(player_agent_chatter_len(&achievements), 0);
}
