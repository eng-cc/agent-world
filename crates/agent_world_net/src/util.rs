use agent_world::runtime::WorldError;
use serde::Serialize;

pub(crate) fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer.self_describe()?;
    canonical_value.serialize(&mut serializer)?;
    Ok(buf)
}
