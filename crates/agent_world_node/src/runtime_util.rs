use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::RuntimeState;

pub(crate) fn lock_state<'a>(
    state: &'a Arc<Mutex<RuntimeState>>,
) -> std::sync::MutexGuard<'a, RuntimeState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(duration_millis_to_i64_saturating)
        .unwrap_or(0)
}

fn duration_millis_to_i64_saturating(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_millis_to_i64_saturating_clamps_on_overflow() {
        assert_eq!(
            duration_millis_to_i64_saturating(Duration::from_secs(u64::MAX)),
            i64::MAX
        );
    }
}
