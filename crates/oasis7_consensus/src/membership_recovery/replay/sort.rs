use std::cmp::Reverse;

use super::MembershipRevocationAlertDeadLetterRecord;

pub(super) fn sort_dead_letter_bucket(
    dead_letters: &[MembershipRevocationAlertDeadLetterRecord],
    indices: &mut [usize],
) {
    indices.sort_by_key(|index| {
        let record = &dead_letters[*index];
        (
            Reverse(record.pending_alert.attempt),
            record.dropped_at_ms,
            *index,
        )
    });
}
