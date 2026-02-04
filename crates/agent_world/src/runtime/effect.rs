//! Effect system types - intents, receipts, and capabilities.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::types::WorldTime;

/// An intent to perform an external effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectIntent {
    pub intent_id: String,
    pub kind: String,
    pub params: JsonValue,
    pub cap_ref: String,
    pub origin: EffectOrigin,
}

/// Receipt returned after an effect is executed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectReceipt {
    pub intent_id: String,
    pub status: String,
    pub payload: JsonValue,
    pub cost_cents: Option<u64>,
    pub signature: Option<ReceiptSignature>,
}

/// Cryptographic signature on a receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptSignature {
    pub algorithm: SignatureAlgorithm,
    pub signature_hex: String,
}

/// Supported signature algorithms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureAlgorithm {
    HmacSha256,
}

/// The origin/source of an effect request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EffectOrigin {
    Reducer { name: String },
    Plan { name: String },
    System,
}

/// Simplified origin kind for policy matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind {
    Reducer,
    Plan,
    System,
}

impl OriginKind {
    pub fn from_origin(origin: &EffectOrigin) -> Self {
        match origin {
            EffectOrigin::Reducer { .. } => OriginKind::Reducer,
            EffectOrigin::Plan { .. } => OriginKind::Plan,
            EffectOrigin::System => OriginKind::System,
        }
    }
}

/// A grant of capability to perform certain effect kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub name: String,
    pub effect_kinds: Vec<String>,
    pub expiry: Option<WorldTime>,
}

impl CapabilityGrant {
    pub fn new(name: impl Into<String>, effect_kinds: Vec<String>) -> Self {
        Self {
            name: name.into(),
            effect_kinds,
            expiry: None,
        }
    }

    pub fn allow_all(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            effect_kinds: vec!["*".to_string()],
            expiry: None,
        }
    }

    pub fn allows(&self, kind: &str) -> bool {
        self.effect_kinds.iter().any(|allowed| {
            allowed == "*"
                || allowed == kind
                || (allowed.ends_with(".*")
                    && kind.starts_with(&allowed[..allowed.len() - 1]))
        })
    }

    pub fn is_expired(&self, now: WorldTime) -> bool {
        match self.expiry {
            Some(expiry) => now > expiry,
            None => false,
        }
    }
}
