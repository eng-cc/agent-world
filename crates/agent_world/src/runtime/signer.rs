//! Receipt signing and verification.

use hmac::{Hmac, Mac};
use serde::Serialize;
use serde_json::Value as JsonValue;
use sha2::Sha256;

use super::effect::{EffectReceipt, ReceiptSignature, SignatureAlgorithm};
use super::error::WorldError;

type HmacSha256 = Hmac<Sha256>;

/// Signer for creating and verifying receipt signatures.
#[derive(Debug, Clone)]
pub struct ReceiptSigner {
    algorithm: SignatureAlgorithm,
    key: Vec<u8>,
}

impl ReceiptSigner {
    pub fn hmac_sha256(key: impl Into<Vec<u8>>) -> Self {
        Self {
            algorithm: SignatureAlgorithm::HmacSha256,
            key: key.into(),
        }
    }

    pub fn sign(&self, receipt: &EffectReceipt) -> Result<ReceiptSignature, WorldError> {
        match self.algorithm {
            SignatureAlgorithm::HmacSha256 => {
                let bytes = receipt_signing_bytes(receipt)?;
                let mut mac = HmacSha256::new_from_slice(&self.key)
                    .map_err(|_| WorldError::SignatureKeyInvalid)?;
                mac.update(&bytes);
                let signature = mac.finalize().into_bytes();
                Ok(ReceiptSignature {
                    algorithm: SignatureAlgorithm::HmacSha256,
                    signature_hex: hex::encode(signature),
                })
            }
        }
    }

    pub fn verify(
        &self,
        receipt: &EffectReceipt,
        signature: &ReceiptSignature,
    ) -> Result<(), WorldError> {
        if signature.algorithm != self.algorithm {
            return Err(WorldError::ReceiptSignatureInvalid {
                intent_id: receipt.intent_id.clone(),
            });
        }
        let expected = self.sign(receipt)?;
        if signature.signature_hex != expected.signature_hex {
            return Err(WorldError::ReceiptSignatureInvalid {
                intent_id: receipt.intent_id.clone(),
            });
        }
        Ok(())
    }
}

fn receipt_signing_bytes(receipt: &EffectReceipt) -> Result<Vec<u8>, WorldError> {
    #[derive(Serialize)]
    struct ReceiptPayload<'a> {
        intent_id: &'a str,
        status: &'a str,
        payload: &'a JsonValue,
        cost_cents: Option<u64>,
    }

    let payload = ReceiptPayload {
        intent_id: &receipt.intent_id,
        status: &receipt.status,
        payload: &receipt.payload,
        cost_cents: receipt.cost_cents,
    };

    Ok(serde_json::to_vec(&payload)?)
}
