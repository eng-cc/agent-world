use agent_world::runtime::blake3_hex;
use agent_world::runtime::{EpochSettlementReport, NodeRewardMintRecord};
use serde::{Deserialize, Serialize};

pub(super) const REWARD_SETTLEMENT_TOPIC_SUFFIX: &str = "reward.settlement";

pub(super) fn reward_settlement_topic(world_id: &str) -> String {
    format!("aw.{world_id}.{}", REWARD_SETTLEMENT_TOPIC_SUFFIX)
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
struct EnvelopeIdentityPayload<'a> {
    world_id: &'a str,
    epoch_index: u64,
    signer_node_id: &'a str,
    report: &'a EpochSettlementReport,
    mint_records: &'a [NodeRewardMintRecord],
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
    let identity = EnvelopeIdentityPayload {
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

    #[test]
    fn settlement_topic_uses_expected_suffix() {
        assert_eq!(reward_settlement_topic("w1"), "aw.w1.reward.settlement");
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
    fn sample_report_shape_stays_compatible() {
        let config = NodePointsConfig::default();
        assert!(config.epoch_pool_points > 0);
    }
}
