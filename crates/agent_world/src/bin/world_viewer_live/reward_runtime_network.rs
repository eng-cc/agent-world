use agent_world::runtime::{
    blake3_hex, EpochSettlementReport, NodePointsRuntimeObservation, NodeRewardMintRecord,
};
use agent_world_node::NodeRole;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

pub(super) const REWARD_SETTLEMENT_TOPIC_SUFFIX: &str = "reward.settlement";
pub(super) const REWARD_OBSERVATION_TOPIC_SUFFIX: &str = "reward.observation";
const REWARD_OBSERVATION_SIGNATURE_PREFIX: &str = "rewardobs:v1:";

pub(super) fn reward_settlement_topic(world_id: &str) -> String {
    format!("aw.{world_id}.{}", REWARD_SETTLEMENT_TOPIC_SUFFIX)
}

pub(super) fn reward_observation_topic(world_id: &str) -> String {
    format!("aw.{world_id}.{}", REWARD_OBSERVATION_TOPIC_SUFFIX)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct RewardSettlementEnvelope {
    pub version: u8,
    pub world_id: String,
    pub epoch_index: u64,
    pub signer_node_id: String,
    pub report: EpochSettlementReport,
    pub mint_records: Vec<NodeRewardMintRecord>,
    pub emitted_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct SettlementEnvelopeIdentityPayload<'a> {
    world_id: &'a str,
    epoch_index: u64,
    signer_node_id: &'a str,
    report: &'a EpochSettlementReport,
    mint_records: &'a [NodeRewardMintRecord],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct RewardObservationPayload {
    pub node_id: String,
    pub role: String,
    pub tick_count: u64,
    pub running: bool,
    pub uptime_checks_passed: u64,
    pub uptime_checks_total: u64,
    pub storage_checks_passed: u64,
    pub storage_checks_total: u64,
    pub staked_storage_bytes: u64,
    pub observed_at_unix_ms: i64,
    pub has_error: bool,
    pub effective_storage_bytes: u64,
    pub storage_challenge_proof_hint: Option<serde_json::Value>,
}

impl RewardObservationPayload {
    pub(super) fn from_observation(observation: NodePointsRuntimeObservation) -> Self {
        Self {
            node_id: observation.node_id,
            role: observation.role.as_str().to_string(),
            tick_count: observation.tick_count,
            running: observation.running,
            uptime_checks_passed: observation.uptime_checks_passed,
            uptime_checks_total: observation.uptime_checks_total,
            storage_checks_passed: observation.storage_checks_passed,
            storage_checks_total: observation.storage_checks_total,
            staked_storage_bytes: observation.staked_storage_bytes,
            observed_at_unix_ms: observation.observed_at_unix_ms,
            has_error: observation.has_error,
            effective_storage_bytes: observation.effective_storage_bytes,
            storage_challenge_proof_hint: observation
                .storage_challenge_proof_hint
                .and_then(|hint| serde_json::to_value(hint).ok()),
        }
    }

    pub(super) fn into_observation(self) -> Result<NodePointsRuntimeObservation, String> {
        let role = self
            .role
            .parse::<NodeRole>()
            .map_err(|_| format!("invalid observation role: {}", self.role))?;
        let storage_challenge_proof_hint = self
            .storage_challenge_proof_hint
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|err| format!("invalid observation proof hint payload: {}", err))
            })
            .transpose()?;
        Ok(NodePointsRuntimeObservation {
            node_id: self.node_id,
            role,
            tick_count: self.tick_count,
            running: self.running,
            uptime_checks_passed: self.uptime_checks_passed,
            uptime_checks_total: self.uptime_checks_total,
            storage_checks_passed: self.storage_checks_passed,
            storage_checks_total: self.storage_checks_total,
            staked_storage_bytes: self.staked_storage_bytes,
            observed_at_unix_ms: self.observed_at_unix_ms,
            has_error: self.has_error,
            effective_storage_bytes: self.effective_storage_bytes,
            storage_challenge_proof_hint,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct RewardObservationTrace {
    pub version: u8,
    pub world_id: String,
    pub observer_node_id: String,
    pub observer_public_key_hex: String,
    pub payload: RewardObservationPayload,
    pub payload_hash: String,
    pub signature: String,
    pub emitted_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ObservationSigningPayload<'a> {
    version: u8,
    world_id: &'a str,
    observer_node_id: &'a str,
    observer_public_key_hex: &'a str,
    payload_hash: &'a str,
    emitted_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ObservationTraceIdentityPayload<'a> {
    version: u8,
    world_id: &'a str,
    observer_node_id: &'a str,
    payload_hash: &'a str,
    emitted_at_unix_ms: i64,
    signature: &'a str,
}

pub(super) fn encode_reward_settlement_envelope(
    envelope: &RewardSettlementEnvelope,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(envelope)
        .map_err(|err| format!("encode settlement envelope failed: {}", err))
}

pub(super) fn decode_reward_settlement_envelope(
    payload: &[u8],
) -> Result<RewardSettlementEnvelope, String> {
    serde_json::from_slice::<RewardSettlementEnvelope>(payload)
        .map_err(|err| format!("decode settlement envelope failed: {}", err))
}

pub(super) fn reward_settlement_envelope_id(
    envelope: &RewardSettlementEnvelope,
) -> Result<String, String> {
    let identity = SettlementEnvelopeIdentityPayload {
        world_id: envelope.world_id.as_str(),
        epoch_index: envelope.epoch_index,
        signer_node_id: envelope.signer_node_id.as_str(),
        report: &envelope.report,
        mint_records: envelope.mint_records.as_slice(),
    };
    let bytes = serde_cbor::to_vec(&identity)
        .map_err(|err| format!("encode settlement envelope identity failed: {}", err))?;
    Ok(blake3_hex(bytes.as_slice()))
}

pub(super) fn encode_reward_observation_trace(
    trace: &RewardObservationTrace,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(trace).map_err(|err| format!("encode observation trace failed: {}", err))
}

pub(super) fn decode_reward_observation_trace(
    payload: &[u8],
) -> Result<RewardObservationTrace, String> {
    serde_json::from_slice::<RewardObservationTrace>(payload)
        .map_err(|err| format!("decode observation trace failed: {}", err))
}

pub(super) fn sign_reward_observation_trace(
    world_id: &str,
    observer_node_id: &str,
    observer_private_key_hex: &str,
    observer_public_key_hex: &str,
    payload: RewardObservationPayload,
    emitted_at_unix_ms: i64,
) -> Result<RewardObservationTrace, String> {
    let signing_key = signing_key_from_hex(observer_private_key_hex)?;
    let expected_public = hex::encode(signing_key.verifying_key().to_bytes());
    if expected_public != observer_public_key_hex {
        return Err("observer public key does not match observer private key".to_string());
    }
    let payload_hash = reward_observation_payload_hash(&payload)?;
    let signing_payload = ObservationSigningPayload {
        version: 1,
        world_id,
        observer_node_id,
        observer_public_key_hex,
        payload_hash: payload_hash.as_str(),
        emitted_at_unix_ms,
    };
    let signing_bytes = serde_cbor::to_vec(&signing_payload)
        .map_err(|err| format!("encode observation signing payload failed: {}", err))?;
    let signature: Signature = signing_key.sign(signing_bytes.as_slice());
    let signature_hex = format!(
        "{}{}",
        REWARD_OBSERVATION_SIGNATURE_PREFIX,
        hex::encode(signature.to_bytes())
    );
    Ok(RewardObservationTrace {
        version: 1,
        world_id: world_id.to_string(),
        observer_node_id: observer_node_id.to_string(),
        observer_public_key_hex: observer_public_key_hex.to_string(),
        payload,
        payload_hash,
        signature: signature_hex,
        emitted_at_unix_ms,
    })
}

pub(super) fn verify_reward_observation_trace(
    trace: &RewardObservationTrace,
) -> Result<(), String> {
    if trace.version != 1 {
        return Err(format!(
            "unsupported observation trace version: {}",
            trace.version
        ));
    }
    let expected_payload_hash = reward_observation_payload_hash(&trace.payload)?;
    if expected_payload_hash != trace.payload_hash {
        return Err("observation payload hash mismatch".to_string());
    }
    let signature_hex = trace
        .signature
        .strip_prefix(REWARD_OBSERVATION_SIGNATURE_PREFIX)
        .ok_or_else(|| "observation signature is not rewardobs:v1".to_string())?;
    let signature_bytes = decode_hex_array::<64>(signature_hex, "observation signature")?;
    let public_bytes = decode_hex_array::<32>(
        trace.observer_public_key_hex.as_str(),
        "observation public key",
    )?;
    let verifying_key = VerifyingKey::from_bytes(&public_bytes)
        .map_err(|err| format!("invalid observation public key bytes: {}", err))?;
    let signing_payload = ObservationSigningPayload {
        version: trace.version,
        world_id: trace.world_id.as_str(),
        observer_node_id: trace.observer_node_id.as_str(),
        observer_public_key_hex: trace.observer_public_key_hex.as_str(),
        payload_hash: trace.payload_hash.as_str(),
        emitted_at_unix_ms: trace.emitted_at_unix_ms,
    };
    let signing_bytes = serde_cbor::to_vec(&signing_payload)
        .map_err(|err| format!("encode observation verify payload failed: {}", err))?;
    verifying_key
        .verify(
            signing_bytes.as_slice(),
            &Signature::from_bytes(&signature_bytes),
        )
        .map_err(|err| format!("verify observation signature failed: {}", err))
}

pub(super) fn reward_observation_trace_id(
    trace: &RewardObservationTrace,
) -> Result<String, String> {
    let identity = ObservationTraceIdentityPayload {
        version: trace.version,
        world_id: trace.world_id.as_str(),
        observer_node_id: trace.observer_node_id.as_str(),
        payload_hash: trace.payload_hash.as_str(),
        emitted_at_unix_ms: trace.emitted_at_unix_ms,
        signature: trace.signature.as_str(),
    };
    let bytes = serde_cbor::to_vec(&identity)
        .map_err(|err| format!("encode observation trace identity failed: {}", err))?;
    Ok(blake3_hex(bytes.as_slice()))
}

fn reward_observation_payload_hash(payload: &RewardObservationPayload) -> Result<String, String> {
    let payload_bytes = serde_cbor::to_vec(payload)
        .map_err(|err| format!("encode observation payload failed: {}", err))?;
    Ok(blake3_hex(payload_bytes.as_slice()))
}

fn signing_key_from_hex(private_key_hex: &str) -> Result<SigningKey, String> {
    let private_bytes = decode_hex_array::<32>(private_key_hex, "observation private key")?;
    Ok(SigningKey::from_bytes(&private_bytes))
}

fn decode_hex_array<const N: usize>(hex_value: &str, field_name: &str) -> Result<[u8; N], String> {
    let raw = hex::decode(hex_value).map_err(|_| format!("{field_name} must be valid hex"))?;
    raw.try_into()
        .map_err(|_| format!("{field_name} must be {N}-byte hex"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::runtime::{NodePointsConfig, NodeSettlement};

    fn sample_envelope() -> RewardSettlementEnvelope {
        RewardSettlementEnvelope {
            version: 1,
            world_id: "w1".to_string(),
            epoch_index: 7,
            signer_node_id: "node-seq".to_string(),
            report: EpochSettlementReport {
                epoch_index: 7,
                pool_points: 100,
                storage_pool_points: 0,
                distributed_points: 100,
                storage_distributed_points: 0,
                total_distributed_points: 100,
                settlements: vec![NodeSettlement {
                    node_id: "node-a".to_string(),
                    obligation_met: true,
                    compute_score: 1.0,
                    storage_score: 0.0,
                    uptime_score: 1.0,
                    reliability_score: 1.0,
                    storage_reward_score: 0.0,
                    rewardable_storage_bytes: 0,
                    penalty_score: 0.0,
                    total_score: 1.0,
                    main_awarded_points: 100,
                    storage_awarded_points: 0,
                    awarded_points: 100,
                    cumulative_points: 100,
                }],
            },
            mint_records: Vec::new(),
            emitted_at_unix_ms: 100,
        }
    }

    fn sample_observation_payload() -> RewardObservationPayload {
        RewardObservationPayload {
            node_id: "node-a".to_string(),
            role: NodeRole::Observer.as_str().to_string(),
            tick_count: 11,
            running: true,
            uptime_checks_passed: 1,
            uptime_checks_total: 1,
            storage_checks_passed: 0,
            storage_checks_total: 0,
            staked_storage_bytes: 0,
            observed_at_unix_ms: 1_000,
            has_error: false,
            effective_storage_bytes: 2048,
            storage_challenge_proof_hint: None,
        }
    }

    fn sample_observation_trace() -> RewardObservationTrace {
        let private_bytes = [42_u8; 32];
        let signing_key = SigningKey::from_bytes(&private_bytes);
        let private_key_hex = hex::encode(signing_key.to_bytes());
        let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
        sign_reward_observation_trace(
            "w1",
            "observer-1",
            private_key_hex.as_str(),
            public_key_hex.as_str(),
            sample_observation_payload(),
            9_999,
        )
        .expect("sign trace")
    }

    #[test]
    fn settlement_topic_uses_expected_suffix() {
        assert_eq!(reward_settlement_topic("w1"), "aw.w1.reward.settlement");
    }

    #[test]
    fn observation_topic_uses_expected_suffix() {
        assert_eq!(reward_observation_topic("w1"), "aw.w1.reward.observation");
    }

    #[test]
    fn settlement_envelope_roundtrip() {
        let envelope = sample_envelope();
        let encoded = encode_reward_settlement_envelope(&envelope).expect("encode");
        let decoded = decode_reward_settlement_envelope(encoded.as_slice()).expect("decode");
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn settlement_envelope_id_is_stable_for_same_payload() {
        let envelope = sample_envelope();
        let id_a = reward_settlement_envelope_id(&envelope).expect("id a");
        let id_b = reward_settlement_envelope_id(&envelope).expect("id b");
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn settlement_envelope_id_changes_with_payload() {
        let mut envelope = sample_envelope();
        let original = reward_settlement_envelope_id(&envelope).expect("id original");
        envelope.epoch_index = 8;
        let changed = reward_settlement_envelope_id(&envelope).expect("id changed");
        assert_ne!(original, changed);
    }

    #[test]
    fn observation_trace_roundtrip() {
        let trace = sample_observation_trace();
        let encoded = encode_reward_observation_trace(&trace).expect("encode");
        let decoded = decode_reward_observation_trace(encoded.as_slice()).expect("decode");
        assert_eq!(decoded, trace);
    }

    #[test]
    fn observation_trace_signature_verifies() {
        let trace = sample_observation_trace();
        verify_reward_observation_trace(&trace).expect("verify");
    }

    #[test]
    fn observation_trace_signature_rejects_tampered_payload() {
        let mut trace = sample_observation_trace();
        trace.payload.tick_count = trace.payload.tick_count.saturating_add(1);
        let err = verify_reward_observation_trace(&trace).expect_err("tamper should fail");
        assert!(err.contains("payload hash mismatch"));
    }

    #[test]
    fn observation_trace_id_is_stable_for_same_payload() {
        let trace = sample_observation_trace();
        let id_a = reward_observation_trace_id(&trace).expect("id a");
        let id_b = reward_observation_trace_id(&trace).expect("id b");
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn observation_payload_roundtrip_to_runtime_observation() {
        let source = NodePointsRuntimeObservation {
            node_id: "node-source".to_string(),
            role: NodeRole::Storage,
            tick_count: 77,
            running: true,
            uptime_checks_passed: 1,
            uptime_checks_total: 1,
            storage_checks_passed: 1,
            storage_checks_total: 1,
            staked_storage_bytes: 1024,
            observed_at_unix_ms: 8_000,
            has_error: false,
            effective_storage_bytes: 1024,
            storage_challenge_proof_hint: None,
        };
        let payload = RewardObservationPayload::from_observation(source.clone());
        let restored = payload.into_observation().expect("restore observation");
        assert_eq!(restored, source);
    }

    #[test]
    fn sample_report_shape_stays_compatible() {
        let config = NodePointsConfig::default();
        assert!(config.epoch_pool_points > 0);
    }
}
