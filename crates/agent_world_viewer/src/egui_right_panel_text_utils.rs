use agent_world::simulator::{WorldEvent, WorldEventKind};

pub(super) fn rejection_event_count(events: &[WorldEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event.kind, WorldEventKind::ActionRejected { .. }))
        .count()
}

pub(super) fn truncate_observe_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars().take(max_chars.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}
