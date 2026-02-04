use serde_json::Value as JsonValue;

use super::World;
use super::super::{
    CausedBy, EffectIntent, EffectOrigin, EffectReceipt, PolicyDecisionRecord, WorldError,
    WorldEventBody, WorldEventId,
};

impl World {
    // ---------------------------------------------------------------------
    // Effect handling
    // ---------------------------------------------------------------------

    pub fn take_next_effect(&mut self) -> Option<EffectIntent> {
        let intent = self.pending_effects.pop_front()?;
        self.inflight_effects
            .insert(intent.intent_id.clone(), intent.clone());
        Some(intent)
    }

    pub fn emit_effect(
        &mut self,
        kind: impl Into<String>,
        params: JsonValue,
        cap_ref: impl Into<String>,
        origin: EffectOrigin,
    ) -> Result<String, WorldError> {
        let intent = self.build_effect_intent(kind, params, cap_ref, origin)?;
        let intent_id = intent.intent_id.clone();
        self.append_event(WorldEventBody::EffectQueued(intent), None)?;
        Ok(intent_id)
    }

    pub(super) fn build_effect_intent(
        &mut self,
        kind: impl Into<String>,
        params: JsonValue,
        cap_ref: impl Into<String>,
        origin: EffectOrigin,
    ) -> Result<EffectIntent, WorldError> {
        let kind = kind.into();
        let cap_ref = cap_ref.into();
        let intent_id = format!("intent-{}", self.next_intent_id);
        self.next_intent_id += 1;

        let intent = EffectIntent {
            intent_id: intent_id.clone(),
            kind: kind.clone(),
            params,
            cap_ref: cap_ref.clone(),
            origin,
        };

        let grant = self
            .capabilities
            .get(&cap_ref)
            .ok_or_else(|| WorldError::CapabilityMissing { cap_ref: cap_ref.clone() })?;

        if grant.is_expired(self.state.time) {
            return Err(WorldError::CapabilityExpired { cap_ref });
        }

        if !grant.allows(&kind) {
            return Err(WorldError::CapabilityNotAllowed { cap_ref, kind });
        }

        let decision = self.policies.decide(&intent);
        let record = PolicyDecisionRecord::from_intent(&intent, decision.clone());
        self.append_event(WorldEventBody::PolicyDecisionRecorded(record), None)?;

        if !decision.is_allowed() {
            return Err(WorldError::PolicyDenied {
                intent_id,
                reason: decision.reason().unwrap_or_else(|| "policy_deny".to_string()),
            });
        }

        Ok(intent)
    }

    pub fn ingest_receipt(&mut self, mut receipt: EffectReceipt) -> Result<WorldEventId, WorldError> {
        let known = self.inflight_effects.contains_key(&receipt.intent_id)
            || self
                .pending_effects
                .iter()
                .any(|intent| intent.intent_id == receipt.intent_id);
        if !known {
            return Err(WorldError::ReceiptUnknownIntent {
                intent_id: receipt.intent_id,
            });
        }

        self.finalize_receipt_signature(&mut receipt)?;
        self.append_event(
            WorldEventBody::ReceiptAppended(receipt.clone()),
            Some(CausedBy::Effect {
                intent_id: receipt.intent_id,
            }),
        )
    }

    fn finalize_receipt_signature(&self, receipt: &mut EffectReceipt) -> Result<(), WorldError> {
        let Some(signer) = &self.receipt_signer else {
            return Ok(());
        };

        if let Some(signature) = &receipt.signature {
            signer.verify(receipt, signature)?;
        } else {
            let signature = signer.sign(receipt)?;
            receipt.signature = Some(signature);
        }

        Ok(())
    }
}
