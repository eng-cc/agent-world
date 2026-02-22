use crate::error::WorldError;
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer.self_describe()?;
    canonical_value.serialize(&mut serializer)?;
    Ok(buf)
}

fn duration_millis_to_i64_saturating(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

pub(crate) fn unix_now_ms_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(duration_millis_to_i64_saturating)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_millis_to_i64_saturating_keeps_in_range_values() {
        assert_eq!(
            duration_millis_to_i64_saturating(Duration::from_millis(1234)),
            1234
        );
    }

    #[test]
    fn duration_millis_to_i64_saturating_clamps_overflow_to_i64_max() {
        assert_eq!(
            duration_millis_to_i64_saturating(Duration::from_secs(u64::MAX)),
            i64::MAX
        );
    }
}
