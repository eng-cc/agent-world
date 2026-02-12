use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::runtime::WorldError;
use agent_world_proto::distributed as proto_distributed;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) fn decode_response<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WorldError> {
    if let Ok(error) = serde_cbor::from_slice::<proto_distributed::ErrorResponse>(bytes) {
        return Err(WorldError::NetworkRequestFailed {
            code: error.code,
            message: error.message,
            retryable: error.retryable,
        });
    }
    Ok(serde_cbor::from_slice(bytes)?)
}

pub(crate) fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer.self_describe()?;
    canonical_value.serialize(&mut serializer)?;
    Ok(buf)
}

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
