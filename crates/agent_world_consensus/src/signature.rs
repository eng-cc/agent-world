use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

use super::distributed::{ActionEnvelope, WorldHeadAnnounce};
use super::error::WorldError;

#[derive(Debug, Clone)]
pub struct HmacSha256Signer {
    key: Vec<u8>,
}

impl HmacSha256Signer {
    pub fn new(key: impl Into<Vec<u8>>) -> Result<Self, WorldError> {
        let key = key.into();
        if key.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "hmac signer key cannot be empty".to_string(),
            });
        }
        Ok(Self { key })
    }

    pub fn sign_action(&self, action: &ActionEnvelope) -> Result<String, WorldError> {
        let mut signable = action.clone();
        signable.signature.clear();
        self.sign_value(&signable)
    }

    pub fn verify_action(&self, action: &ActionEnvelope) -> Result<(), WorldError> {
        if action.signature.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "action signature missing for {}:{}",
                    action.world_id, action.action_id
                ),
            });
        }

        let mut signable = action.clone();
        let signature_hex = signable.signature.clone();
        signable.signature.clear();
        self.verify_value(&signable, &signature_hex)
    }

    pub fn sign_head(&self, head: &WorldHeadAnnounce) -> Result<String, WorldError> {
        let mut signable = head.clone();
        signable.signature.clear();
        self.sign_value(&signable)
    }

    pub fn verify_head(&self, head: &WorldHeadAnnounce) -> Result<(), WorldError> {
        if head.signature.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "head signature missing for {}@{}",
                    head.world_id, head.height
                ),
            });
        }

        let mut signable = head.clone();
        let signature_hex = signable.signature.clone();
        signable.signature.clear();
        self.verify_value(&signable, &signature_hex)
    }

    fn sign_value<T: Serialize>(&self, value: &T) -> Result<String, WorldError> {
        let bytes = to_canonical_cbor(value)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.key).map_err(|_| {
            WorldError::DistributedValidationFailed {
                reason: "failed to initialize hmac signer".to_string(),
            }
        })?;
        mac.update(&bytes);
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    fn verify_value<T: Serialize>(&self, value: &T, signature_hex: &str) -> Result<(), WorldError> {
        let bytes = to_canonical_cbor(value)?;
        let signature =
            hex::decode(signature_hex).map_err(|_| WorldError::DistributedValidationFailed {
                reason: "signature is not valid hex".to_string(),
            })?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.key).map_err(|_| {
            WorldError::DistributedValidationFailed {
                reason: "failed to initialize hmac verifier".to_string(),
            }
        })?;
        mac.update(&bytes);
        mac.verify_slice(&signature)
            .map_err(|_| WorldError::DistributedValidationFailed {
                reason: "signature verification failed".to_string(),
            })
    }
}

fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    super::util::to_canonical_cbor(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_action() -> ActionEnvelope {
        ActionEnvelope {
            world_id: "w1".to_string(),
            action_id: "a1".to_string(),
            actor_id: "agent-1".to_string(),
            action_kind: "move".to_string(),
            payload_cbor: vec![1, 2],
            payload_hash: "hash".to_string(),
            nonce: 7,
            timestamp_ms: 42,
            signature: String::new(),
        }
    }

    fn demo_head() -> WorldHeadAnnounce {
        WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 3,
            block_hash: "b3".to_string(),
            state_root: "s3".to_string(),
            timestamp_ms: 99,
            signature: String::new(),
        }
    }

    #[test]
    fn hmac_sign_and_verify_action_roundtrip() {
        let signer = HmacSha256Signer::new(b"demo-key".to_vec()).expect("signer");
        let mut action = demo_action();
        action.signature = signer.sign_action(&action).expect("sign");
        signer.verify_action(&action).expect("verify");
    }

    #[test]
    fn hmac_sign_and_verify_head_roundtrip() {
        let signer = HmacSha256Signer::new(b"demo-key".to_vec()).expect("signer");
        let mut head = demo_head();
        head.signature = signer.sign_head(&head).expect("sign");
        signer.verify_head(&head).expect("verify");
    }

    #[test]
    fn hmac_verify_rejects_tampered_action() {
        let signer = HmacSha256Signer::new(b"demo-key".to_vec()).expect("signer");
        let mut action = demo_action();
        action.signature = signer.sign_action(&action).expect("sign");
        action.payload_hash = "tampered".to_string();

        let result = signer.verify_action(&action);
        assert!(matches!(
            result,
            Err(WorldError::DistributedValidationFailed { .. })
        ));
    }
}
