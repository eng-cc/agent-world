use std::collections::HashSet;

use super::error::WorldError;

pub(super) fn parse_ed25519_public_key_bytes(
    public_key_hex: &str,
    field: &str,
) -> Result<[u8; 32], WorldError> {
    let normalized = public_key_hex.trim();
    if normalized.is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("{field} cannot be empty"),
        });
    }
    let bytes = hex::decode(normalized).map_err(|_| WorldError::DistributedValidationFailed {
        reason: format!("{field} must be valid hex"),
    })?;
    bytes
        .try_into()
        .map_err(|_| WorldError::DistributedValidationFailed {
            reason: format!("{field} must be 32-byte hex"),
        })
}

pub(super) fn normalize_ed25519_public_key_hex(
    public_key_hex: &str,
    field: &str,
) -> Result<String, WorldError> {
    let public_key_bytes = parse_ed25519_public_key_bytes(public_key_hex, field)?;
    Ok(hex::encode(public_key_bytes))
}

pub(super) fn normalize_ed25519_public_key_allowlist(
    keys: &[String],
    entry_field_prefix: &str,
    allowlist_field: &str,
) -> Result<Option<HashSet<String>>, WorldError> {
    if keys.is_empty() {
        return Ok(None);
    }
    let mut normalized = HashSet::with_capacity(keys.len());
    for (index, key) in keys.iter().enumerate() {
        let field = format!("{entry_field_prefix}[{index}]");
        let key = normalize_ed25519_public_key_hex(key, field.as_str())?;
        if !normalized.insert(key.clone()) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("{allowlist_field} contains duplicate signer public key: {key}"),
            });
        }
    }
    Ok(Some(normalized))
}
