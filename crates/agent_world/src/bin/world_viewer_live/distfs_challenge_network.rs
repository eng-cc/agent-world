use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use agent_world::runtime::{blake3_hex, LocalCasStore};
use agent_world_distfs::{
    storage_challenge_receipt_to_proof_semantics, verify_storage_challenge_receipt,
    StorageChallenge, StorageChallengeProbeReport, StorageChallengeReceipt,
    StorageChallengeRequest, STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1,
    STORAGE_CHALLENGE_VERSION,
};
use agent_world_node::NodeRole;
use agent_world_proto::distributed::{
    StorageChallengeFailureReason, StorageChallengeProofSemantics, StorageChallengeSampleSource,
};
use agent_world_proto::distributed_net::{DistributedNetwork, NetworkSubscription};
use agent_world_proto::world_error::WorldError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use super::DistfsProbeRuntimeConfig;

pub(super) const DISTFS_CHALLENGE_REQUEST_TOPIC_SUFFIX: &str = "distfs.challenge.request";
pub(super) const DISTFS_CHALLENGE_PROOF_TOPIC_SUFFIX: &str = "distfs.challenge.proof";
const DISTFS_CHALLENGE_REQUEST_SIGNATURE_PREFIX: &str = "distfschreq:v1:";
const DISTFS_CHALLENGE_PROOF_SIGNATURE_PREFIX: &str = "distfschproof:v1:";

pub(super) fn distfs_challenge_request_topic(world_id: &str) -> String {
    format!("aw.{world_id}.{}", DISTFS_CHALLENGE_REQUEST_TOPIC_SUFFIX)
}

pub(super) fn distfs_challenge_proof_topic(world_id: &str) -> String {
    format!("aw.{world_id}.{}", DISTFS_CHALLENGE_PROOF_TOPIC_SUFFIX)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct DistfsChallengeRequestEnvelope {
    pub version: u8,
    pub world_id: String,
    pub challenger_node_id: String,
    pub challenger_public_key_hex: String,
    pub target_node_id: String,
    pub challenge: StorageChallenge,
    pub emitted_at_unix_ms: i64,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct DistfsChallengeProofEnvelope {
    pub version: u8,
    pub world_id: String,
    pub responder_node_id: String,
    pub responder_public_key_hex: String,
    pub challenge: StorageChallenge,
    pub receipt: StorageChallengeReceipt,
    pub emitted_at_unix_ms: i64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize)]
struct DistfsChallengeRequestSigningPayload<'a> {
    version: u8,
    world_id: &'a str,
    challenger_node_id: &'a str,
    challenger_public_key_hex: &'a str,
    target_node_id: &'a str,
    challenge: &'a StorageChallenge,
    emitted_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct DistfsChallengeProofSigningPayload<'a> {
    version: u8,
    world_id: &'a str,
    responder_node_id: &'a str,
    responder_public_key_hex: &'a str,
    challenge: &'a StorageChallenge,
    receipt: &'a StorageChallengeReceipt,
    emitted_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct DistfsChallengeRequestIdentityPayload<'a> {
    version: u8,
    world_id: &'a str,
    challenge_id: &'a str,
    challenger_node_id: &'a str,
    target_node_id: &'a str,
    signature: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct DistfsChallengeProofIdentityPayload<'a> {
    version: u8,
    world_id: &'a str,
    challenge_id: &'a str,
    responder_node_id: &'a str,
    signature: &'a str,
}

pub(super) fn encode_distfs_challenge_request(
    envelope: &DistfsChallengeRequestEnvelope,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(envelope).map_err(|err| format!("encode distfs request failed: {}", err))
}

pub(super) fn decode_distfs_challenge_request(
    payload: &[u8],
) -> Result<DistfsChallengeRequestEnvelope, String> {
    serde_json::from_slice::<DistfsChallengeRequestEnvelope>(payload)
        .map_err(|err| format!("decode distfs request failed: {}", err))
}

pub(super) fn encode_distfs_challenge_proof(
    envelope: &DistfsChallengeProofEnvelope,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(envelope).map_err(|err| format!("encode distfs proof failed: {}", err))
}

pub(super) fn decode_distfs_challenge_proof(
    payload: &[u8],
) -> Result<DistfsChallengeProofEnvelope, String> {
    serde_json::from_slice::<DistfsChallengeProofEnvelope>(payload)
        .map_err(|err| format!("decode distfs proof failed: {}", err))
}

pub(super) fn sign_distfs_challenge_request(
    world_id: &str,
    challenger_node_id: &str,
    challenger_private_key_hex: &str,
    challenger_public_key_hex: &str,
    target_node_id: &str,
    challenge: StorageChallenge,
    emitted_at_unix_ms: i64,
) -> Result<DistfsChallengeRequestEnvelope, String> {
    let signing_key = signing_key_from_hex(challenger_private_key_hex)?;
    let expected_public = hex::encode(signing_key.verifying_key().to_bytes());
    if expected_public != challenger_public_key_hex {
        return Err("distfs request signer public key does not match private key".to_string());
    }
    let payload = DistfsChallengeRequestSigningPayload {
        version: 1,
        world_id,
        challenger_node_id,
        challenger_public_key_hex,
        target_node_id,
        challenge: &challenge,
        emitted_at_unix_ms,
    };
    let signing_bytes = serde_cbor::to_vec(&payload)
        .map_err(|err| format!("encode distfs request signing payload failed: {}", err))?;
    let signature: Signature = signing_key.sign(signing_bytes.as_slice());
    let signature = format!(
        "{}{}",
        DISTFS_CHALLENGE_REQUEST_SIGNATURE_PREFIX,
        hex::encode(signature.to_bytes())
    );
    Ok(DistfsChallengeRequestEnvelope {
        version: 1,
        world_id: world_id.to_string(),
        challenger_node_id: challenger_node_id.to_string(),
        challenger_public_key_hex: challenger_public_key_hex.to_string(),
        target_node_id: target_node_id.to_string(),
        challenge,
        emitted_at_unix_ms,
        signature,
    })
}

pub(super) fn verify_distfs_challenge_request(
    envelope: &DistfsChallengeRequestEnvelope,
) -> Result<(), String> {
    if envelope.version != 1 {
        return Err(format!(
            "unsupported distfs request version: {}",
            envelope.version
        ));
    }
    if envelope.challenge.version != STORAGE_CHALLENGE_VERSION {
        return Err(format!(
            "unsupported distfs challenge version: {}",
            envelope.challenge.version
        ));
    }
    if envelope.challenge.node_id != envelope.target_node_id {
        return Err("distfs request target_node_id mismatch with challenge.node_id".to_string());
    }
    let signature_hex = envelope
        .signature
        .strip_prefix(DISTFS_CHALLENGE_REQUEST_SIGNATURE_PREFIX)
        .ok_or_else(|| "distfs request signature is not distfschreq:v1".to_string())?;
    let signature_bytes = decode_hex_array::<64>(signature_hex, "distfs request signature")?;
    let public_bytes = decode_hex_array::<32>(
        envelope.challenger_public_key_hex.as_str(),
        "distfs request signer public key",
    )?;
    let verifying_key = VerifyingKey::from_bytes(&public_bytes)
        .map_err(|err| format!("invalid distfs request signer public key bytes: {}", err))?;
    let payload = DistfsChallengeRequestSigningPayload {
        version: envelope.version,
        world_id: envelope.world_id.as_str(),
        challenger_node_id: envelope.challenger_node_id.as_str(),
        challenger_public_key_hex: envelope.challenger_public_key_hex.as_str(),
        target_node_id: envelope.target_node_id.as_str(),
        challenge: &envelope.challenge,
        emitted_at_unix_ms: envelope.emitted_at_unix_ms,
    };
    let signing_bytes = serde_cbor::to_vec(&payload)
        .map_err(|err| format!("encode distfs request verify payload failed: {}", err))?;
    verifying_key
        .verify(
            signing_bytes.as_slice(),
            &Signature::from_bytes(&signature_bytes),
        )
        .map_err(|err| format!("verify distfs request signature failed: {}", err))
}

pub(super) fn sign_distfs_challenge_proof(
    world_id: &str,
    responder_node_id: &str,
    responder_private_key_hex: &str,
    responder_public_key_hex: &str,
    challenge: StorageChallenge,
    receipt: StorageChallengeReceipt,
    emitted_at_unix_ms: i64,
) -> Result<DistfsChallengeProofEnvelope, String> {
    let signing_key = signing_key_from_hex(responder_private_key_hex)?;
    let expected_public = hex::encode(signing_key.verifying_key().to_bytes());
    if expected_public != responder_public_key_hex {
        return Err("distfs proof signer public key does not match private key".to_string());
    }
    let payload = DistfsChallengeProofSigningPayload {
        version: 1,
        world_id,
        responder_node_id,
        responder_public_key_hex,
        challenge: &challenge,
        receipt: &receipt,
        emitted_at_unix_ms,
    };
    let signing_bytes = serde_cbor::to_vec(&payload)
        .map_err(|err| format!("encode distfs proof signing payload failed: {}", err))?;
    let signature: Signature = signing_key.sign(signing_bytes.as_slice());
    let signature = format!(
        "{}{}",
        DISTFS_CHALLENGE_PROOF_SIGNATURE_PREFIX,
        hex::encode(signature.to_bytes())
    );
    Ok(DistfsChallengeProofEnvelope {
        version: 1,
        world_id: world_id.to_string(),
        responder_node_id: responder_node_id.to_string(),
        responder_public_key_hex: responder_public_key_hex.to_string(),
        challenge,
        receipt,
        emitted_at_unix_ms,
        signature,
    })
}

pub(super) fn verify_distfs_challenge_proof(
    envelope: &DistfsChallengeProofEnvelope,
) -> Result<(), String> {
    if envelope.version != 1 {
        return Err(format!(
            "unsupported distfs proof version: {}",
            envelope.version
        ));
    }
    if envelope.challenge.version != STORAGE_CHALLENGE_VERSION {
        return Err(format!(
            "unsupported distfs challenge version in proof: {}",
            envelope.challenge.version
        ));
    }
    if envelope.receipt.challenge_id != envelope.challenge.challenge_id {
        return Err("distfs proof receipt challenge_id mismatch".to_string());
    }
    if envelope.receipt.node_id != envelope.challenge.node_id {
        return Err("distfs proof receipt node_id mismatch".to_string());
    }
    let signature_hex = envelope
        .signature
        .strip_prefix(DISTFS_CHALLENGE_PROOF_SIGNATURE_PREFIX)
        .ok_or_else(|| "distfs proof signature is not distfschproof:v1".to_string())?;
    let signature_bytes = decode_hex_array::<64>(signature_hex, "distfs proof signature")?;
    let public_bytes = decode_hex_array::<32>(
        envelope.responder_public_key_hex.as_str(),
        "distfs proof signer public key",
    )?;
    let verifying_key = VerifyingKey::from_bytes(&public_bytes)
        .map_err(|err| format!("invalid distfs proof signer public key bytes: {}", err))?;
    let payload = DistfsChallengeProofSigningPayload {
        version: envelope.version,
        world_id: envelope.world_id.as_str(),
        responder_node_id: envelope.responder_node_id.as_str(),
        responder_public_key_hex: envelope.responder_public_key_hex.as_str(),
        challenge: &envelope.challenge,
        receipt: &envelope.receipt,
        emitted_at_unix_ms: envelope.emitted_at_unix_ms,
    };
    let signing_bytes = serde_cbor::to_vec(&payload)
        .map_err(|err| format!("encode distfs proof verify payload failed: {}", err))?;
    verifying_key
        .verify(
            signing_bytes.as_slice(),
            &Signature::from_bytes(&signature_bytes),
        )
        .map_err(|err| format!("verify distfs proof signature failed: {}", err))
}

pub(super) fn distfs_challenge_request_id(
    envelope: &DistfsChallengeRequestEnvelope,
) -> Result<String, String> {
    let identity = DistfsChallengeRequestIdentityPayload {
        version: envelope.version,
        world_id: envelope.world_id.as_str(),
        challenge_id: envelope.challenge.challenge_id.as_str(),
        challenger_node_id: envelope.challenger_node_id.as_str(),
        target_node_id: envelope.target_node_id.as_str(),
        signature: envelope.signature.as_str(),
    };
    let bytes = serde_cbor::to_vec(&identity)
        .map_err(|err| format!("encode distfs request identity failed: {}", err))?;
    Ok(blake3_hex(bytes.as_slice()))
}

pub(super) fn distfs_challenge_proof_id(
    envelope: &DistfsChallengeProofEnvelope,
) -> Result<String, String> {
    let identity = DistfsChallengeProofIdentityPayload {
        version: envelope.version,
        world_id: envelope.world_id.as_str(),
        challenge_id: envelope.challenge.challenge_id.as_str(),
        responder_node_id: envelope.responder_node_id.as_str(),
        signature: envelope.signature.as_str(),
    };
    let bytes = serde_cbor::to_vec(&identity)
        .map_err(|err| format!("encode distfs proof identity failed: {}", err))?;
    Ok(blake3_hex(bytes.as_slice()))
}

pub(super) fn storage_proof_hint_value_from_semantics(
    semantics: &StorageChallengeProofSemantics,
) -> serde_json::Value {
    serde_json::json!({
        "sample_source": match semantics.sample_source {
            StorageChallengeSampleSource::LocalStoreIndex => "local_store_index",
            StorageChallengeSampleSource::ReplicationCommit => "replication_commit",
            StorageChallengeSampleSource::GossipReplicaHint | StorageChallengeSampleSource::Unknown => "unknown",
        },
        "sample_reference": semantics.sample_reference,
        "failure_reason": semantics.failure_reason.map(|reason| match reason {
            StorageChallengeFailureReason::MissingSample => "missing_sample",
            StorageChallengeFailureReason::HashMismatch => "hash_mismatch",
            StorageChallengeFailureReason::Timeout => "timeout",
            StorageChallengeFailureReason::ReadIoError => "read_io_error",
            StorageChallengeFailureReason::SignatureInvalid | StorageChallengeFailureReason::Unknown => "unknown",
        }),
        "proof_kind_hint": semantics.proof_kind_hint,
        "vrf_seed_hint": semantics.vrf_seed_hint,
        "post_commitment_hint": semantics.post_commitment_hint,
    })
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DistfsChallengeNetworkTickReport {
    pub mode: String,
    pub request_topic: String,
    pub proof_topic: String,
    pub known_storage_targets: Vec<String>,
    pub issued_challenge_ids: Vec<String>,
    pub answered_challenge_ids: Vec<String>,
    pub accepted_proof_ids: Vec<String>,
    pub timed_out_challenge_ids: Vec<String>,
    pub probe_report: Option<StorageChallengeProbeReport>,
}

impl DistfsChallengeNetworkTickReport {
    pub(super) fn should_fallback_local(&self) -> bool {
        self.mode == "fallback_local"
    }
}

#[derive(Debug, Clone)]
struct PendingChallenge {
    challenge: StorageChallenge,
    target_node_id: String,
}

pub(super) struct DistfsChallengeNetworkDriver {
    world_id: String,
    local_node_id: String,
    signer_private_key_hex: String,
    signer_public_key_hex: String,
    network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync>,
    request_topic: String,
    proof_topic: String,
    request_subscription: NetworkSubscription,
    proof_subscription: NetworkSubscription,
    store: LocalCasStore,
    probe_config: DistfsProbeRuntimeConfig,
    known_node_roles: BTreeMap<String, NodeRole>,
    pending_challenges: BTreeMap<String, PendingChallenge>,
    processed_request_ids: BTreeSet<String>,
    processed_proof_ids: BTreeSet<String>,
    issue_cursor: u64,
}

impl DistfsChallengeNetworkDriver {
    pub(super) fn new(
        world_id: &str,
        local_node_id: &str,
        signer_private_key_hex: &str,
        signer_public_key_hex: &str,
        storage_root: std::path::PathBuf,
        probe_config: DistfsProbeRuntimeConfig,
        network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync>,
    ) -> Result<Self, String> {
        let request_topic = distfs_challenge_request_topic(world_id);
        let proof_topic = distfs_challenge_proof_topic(world_id);
        let request_subscription = network
            .subscribe(request_topic.as_str())
            .map_err(|err| format!("subscribe distfs request topic failed: {:?}", err))?;
        let proof_subscription = network
            .subscribe(proof_topic.as_str())
            .map_err(|err| format!("subscribe distfs proof topic failed: {:?}", err))?;
        Ok(Self {
            world_id: world_id.to_string(),
            local_node_id: local_node_id.to_string(),
            signer_private_key_hex: signer_private_key_hex.to_string(),
            signer_public_key_hex: signer_public_key_hex.to_string(),
            network,
            request_topic,
            proof_topic,
            request_subscription,
            proof_subscription,
            store: LocalCasStore::new(storage_root),
            probe_config,
            known_node_roles: BTreeMap::new(),
            pending_challenges: BTreeMap::new(),
            processed_request_ids: BTreeSet::new(),
            processed_proof_ids: BTreeSet::new(),
            issue_cursor: 0,
        })
    }

    pub(super) fn register_observation_role(&mut self, node_id: &str, role: NodeRole) {
        if node_id.trim().is_empty() {
            return;
        }
        self.known_node_roles.insert(node_id.to_string(), role);
    }

    pub(super) fn tick(&mut self, observed_at_unix_ms: i64) -> DistfsChallengeNetworkTickReport {
        let mut passed_checks = 0_u64;
        let mut failed_checks = 0_u64;
        let mut failure_reasons = BTreeMap::new();
        let mut latest_proof_semantics: Option<StorageChallengeProofSemantics> = None;
        let mut issued_challenge_ids = Vec::new();
        let mut answered_challenge_ids = Vec::new();
        let mut accepted_proof_ids = Vec::new();
        let mut timed_out_challenge_ids = Vec::new();

        for payload in self.proof_subscription.drain() {
            let proof = match decode_distfs_challenge_proof(payload.as_slice()) {
                Ok(proof) => proof,
                Err(err) => {
                    eprintln!("reward runtime decode distfs proof failed: {err}");
                    continue;
                }
            };
            if proof.version != 1 || proof.world_id != self.world_id {
                continue;
            }
            if let Err(err) = verify_distfs_challenge_proof(&proof) {
                eprintln!("reward runtime verify distfs proof failed: {err}");
                continue;
            }
            let proof_id = match distfs_challenge_proof_id(&proof) {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("reward runtime hash distfs proof failed: {err}");
                    continue;
                }
            };
            if self.processed_proof_ids.contains(proof_id.as_str()) {
                continue;
            }
            self.processed_proof_ids.insert(proof_id.clone());

            let Some(pending) = self.pending_challenges.remove(proof.challenge.challenge_id.as_str())
            else {
                continue;
            };
            if pending.target_node_id != proof.responder_node_id {
                failed_checks = failed_checks.saturating_add(1);
                increment_failure_reason(&mut failure_reasons, StorageChallengeFailureReason::Unknown);
                accepted_proof_ids.push(proof_id);
                continue;
            }
            match verify_storage_challenge_receipt(
                &pending.challenge,
                &proof.receipt,
                self.probe_config.allowed_clock_skew_ms,
            ) {
                Ok(()) => {
                    passed_checks = passed_checks.saturating_add(1);
                    latest_proof_semantics = Some(storage_challenge_receipt_to_proof_semantics(
                        &pending.challenge,
                        &proof.receipt,
                    ));
                }
                Err(_) => {
                    failed_checks = failed_checks.saturating_add(1);
                    let reason = classify_receipt_failure_reason(
                        &pending.challenge,
                        &proof.receipt,
                        self.probe_config.allowed_clock_skew_ms,
                    );
                    increment_failure_reason(&mut failure_reasons, reason);
                }
            }
            accepted_proof_ids.push(proof_id);
        }

        for payload in self.request_subscription.drain() {
            let request = match decode_distfs_challenge_request(payload.as_slice()) {
                Ok(request) => request,
                Err(err) => {
                    eprintln!("reward runtime decode distfs request failed: {err}");
                    continue;
                }
            };
            if request.version != 1 || request.world_id != self.world_id {
                continue;
            }
            if let Err(err) = verify_distfs_challenge_request(&request) {
                eprintln!("reward runtime verify distfs request failed: {err}");
                continue;
            }
            let request_id = match distfs_challenge_request_id(&request) {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("reward runtime hash distfs request failed: {err}");
                    continue;
                }
            };
            if self.processed_request_ids.contains(request_id.as_str()) {
                continue;
            }
            self.processed_request_ids.insert(request_id);
            if request.target_node_id != self.local_node_id {
                continue;
            }

            let receipt = match self
                .store
                .answer_storage_challenge(&request.challenge, observed_at_unix_ms)
            {
                Ok(receipt) => receipt,
                Err(err) => build_failed_receipt(
                    &request.challenge,
                    observed_at_unix_ms,
                    classify_world_error_reason(&err),
                ),
            };
            let proof = match sign_distfs_challenge_proof(
                self.world_id.as_str(),
                self.local_node_id.as_str(),
                self.signer_private_key_hex.as_str(),
                self.signer_public_key_hex.as_str(),
                request.challenge.clone(),
                receipt,
                observed_at_unix_ms,
            ) {
                Ok(proof) => proof,
                Err(err) => {
                    eprintln!("reward runtime sign distfs proof failed: {err}");
                    continue;
                }
            };
            match encode_distfs_challenge_proof(&proof) {
                Ok(payload) => {
                    if let Err(err) = self
                        .network
                        .publish(self.proof_topic.as_str(), payload.as_slice())
                    {
                        eprintln!("reward runtime publish distfs proof failed: {:?}", err);
                        continue;
                    }
                    answered_challenge_ids.push(proof.challenge.challenge_id.clone());
                }
                Err(err) => {
                    eprintln!("reward runtime encode distfs proof failed: {err}");
                }
            }
        }

        let mut expired = Vec::new();
        for (challenge_id, pending) in &self.pending_challenges {
            let timeout_cutoff = pending
                .challenge
                .expires_at_unix_ms
                .saturating_add(self.probe_config.allowed_clock_skew_ms);
            if observed_at_unix_ms > timeout_cutoff {
                expired.push(challenge_id.clone());
            }
        }
        for challenge_id in expired {
            self.pending_challenges.remove(challenge_id.as_str());
            failed_checks = failed_checks.saturating_add(1);
            increment_failure_reason(&mut failure_reasons, StorageChallengeFailureReason::Timeout);
            timed_out_challenge_ids.push(challenge_id);
        }

        let mut known_storage_targets: Vec<String> = self
            .known_node_roles
            .iter()
            .filter_map(|(node_id, role)| {
                if *role == NodeRole::Storage && node_id != &self.local_node_id {
                    Some(node_id.clone())
                } else {
                    None
                }
            })
            .collect();
        known_storage_targets.sort();

        if !known_storage_targets.is_empty() {
            match self.store.list_blob_hashes() {
                Ok(hashes) => {
                    if !hashes.is_empty() {
                        let max_round = self.probe_config.challenges_per_tick.max(1) as usize;
                        for idx in 0..max_round {
                            let target = known_storage_targets
                                [(self.issue_cursor as usize + idx) % known_storage_targets.len()]
                                .clone();
                            let content_hash =
                                hashes[(self.issue_cursor as usize + idx) % hashes.len()].clone();
                            let challenge_id = build_network_challenge_id(
                                self.local_node_id.as_str(),
                                target.as_str(),
                                observed_at_unix_ms,
                                self.issue_cursor,
                                content_hash.as_str(),
                            );
                            let request = StorageChallengeRequest {
                                challenge_id: challenge_id.clone(),
                                world_id: self.world_id.clone(),
                                node_id: target.clone(),
                                content_hash,
                                max_sample_bytes: self.probe_config.max_sample_bytes,
                                issued_at_unix_ms: observed_at_unix_ms,
                                challenge_ttl_ms: self.probe_config.challenge_ttl_ms,
                                vrf_seed: format!(
                                    "{}:{}:{}:{}",
                                    self.world_id, self.local_node_id, observed_at_unix_ms, self.issue_cursor
                                ),
                            };
                            let challenge = match self.store.issue_storage_challenge(&request) {
                                Ok(challenge) => challenge,
                                Err(err) => {
                                    failed_checks = failed_checks.saturating_add(1);
                                    increment_failure_reason(
                                        &mut failure_reasons,
                                        classify_world_error_reason(&err),
                                    );
                                    self.issue_cursor = self.issue_cursor.saturating_add(1);
                                    continue;
                                }
                            };
                            let envelope = match sign_distfs_challenge_request(
                                self.world_id.as_str(),
                                self.local_node_id.as_str(),
                                self.signer_private_key_hex.as_str(),
                                self.signer_public_key_hex.as_str(),
                                target.as_str(),
                                challenge.clone(),
                                observed_at_unix_ms,
                            ) {
                                Ok(envelope) => envelope,
                                Err(err) => {
                                    eprintln!("reward runtime sign distfs request failed: {err}");
                                    failed_checks = failed_checks.saturating_add(1);
                                    increment_failure_reason(
                                        &mut failure_reasons,
                                        StorageChallengeFailureReason::SignatureInvalid,
                                    );
                                    self.issue_cursor = self.issue_cursor.saturating_add(1);
                                    continue;
                                }
                            };
                            match encode_distfs_challenge_request(&envelope) {
                                Ok(payload) => {
                                    if let Err(err) = self
                                        .network
                                        .publish(self.request_topic.as_str(), payload.as_slice())
                                    {
                                        eprintln!(
                                            "reward runtime publish distfs request failed: {:?}",
                                            err
                                        );
                                        failed_checks = failed_checks.saturating_add(1);
                                        increment_failure_reason(
                                            &mut failure_reasons,
                                            StorageChallengeFailureReason::Unknown,
                                        );
                                    } else {
                                        self.pending_challenges.insert(
                                            challenge.challenge_id.clone(),
                                            PendingChallenge {
                                                challenge,
                                                target_node_id: target,
                                            },
                                        );
                                        issued_challenge_ids.push(challenge_id);
                                    }
                                }
                                Err(err) => {
                                    eprintln!("reward runtime encode distfs request failed: {err}");
                                    failed_checks = failed_checks.saturating_add(1);
                                    increment_failure_reason(
                                        &mut failure_reasons,
                                        StorageChallengeFailureReason::Unknown,
                                    );
                                }
                            }
                            self.issue_cursor = self.issue_cursor.saturating_add(1);
                        }
                    }
                }
                Err(err) => {
                    failed_checks = failed_checks.saturating_add(1);
                    increment_failure_reason(&mut failure_reasons, classify_world_error_reason(&err));
                }
            }
        }

        let network_mode = !known_storage_targets.is_empty() || !self.pending_challenges.is_empty();
        let probe_report = if network_mode {
            Some(StorageChallengeProbeReport {
                node_id: self.local_node_id.clone(),
                world_id: self.world_id.clone(),
                observed_at_unix_ms,
                total_checks: passed_checks.saturating_add(failed_checks),
                passed_checks,
                failed_checks,
                failure_reasons,
                latest_proof_semantics,
            })
        } else {
            None
        };

        DistfsChallengeNetworkTickReport {
            mode: if network_mode {
                "network".to_string()
            } else {
                "fallback_local".to_string()
            },
            request_topic: self.request_topic.clone(),
            proof_topic: self.proof_topic.clone(),
            known_storage_targets,
            issued_challenge_ids,
            answered_challenge_ids,
            accepted_proof_ids,
            timed_out_challenge_ids,
            probe_report,
        }
    }
}

fn build_network_challenge_id(
    local_node_id: &str,
    target_node_id: &str,
    observed_at_unix_ms: i64,
    cursor: u64,
    content_hash: &str,
) -> String {
    let prefix_len = content_hash.len().min(12);
    let prefix = &content_hash[..prefix_len];
    format!(
        "netprobe:{}:{}:{}:{}:{}",
        local_node_id, target_node_id, observed_at_unix_ms, cursor, prefix
    )
}

fn build_failed_receipt(
    challenge: &StorageChallenge,
    responded_at_unix_ms: i64,
    reason: StorageChallengeFailureReason,
) -> StorageChallengeReceipt {
    StorageChallengeReceipt {
        version: STORAGE_CHALLENGE_VERSION,
        challenge_id: challenge.challenge_id.clone(),
        node_id: challenge.node_id.clone(),
        content_hash: challenge.content_hash.clone(),
        sample_offset: challenge.sample_offset,
        sample_size_bytes: challenge.sample_size_bytes,
        sample_hash: challenge.expected_sample_hash.clone(),
        responded_at_unix_ms,
        sample_source: StorageChallengeSampleSource::ReplicationCommit,
        failure_reason: Some(reason),
        proof_kind: STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1.to_string(),
    }
}

fn increment_failure_reason(
    map: &mut BTreeMap<String, u64>,
    reason: StorageChallengeFailureReason,
) {
    let key = failure_reason_key(reason).to_string();
    let count = map.entry(key).or_insert(0);
    *count = count.saturating_add(1);
}

fn failure_reason_key(reason: StorageChallengeFailureReason) -> &'static str {
    match reason {
        StorageChallengeFailureReason::MissingSample => "MISSING_SAMPLE",
        StorageChallengeFailureReason::HashMismatch => "HASH_MISMATCH",
        StorageChallengeFailureReason::Timeout => "TIMEOUT",
        StorageChallengeFailureReason::ReadIoError => "READ_IO_ERROR",
        StorageChallengeFailureReason::SignatureInvalid => "SIGNATURE_INVALID",
        StorageChallengeFailureReason::Unknown => "UNKNOWN",
    }
}

fn classify_world_error_reason(error: &WorldError) -> StorageChallengeFailureReason {
    match error {
        WorldError::BlobNotFound { .. } => StorageChallengeFailureReason::MissingSample,
        WorldError::BlobHashMismatch { .. } | WorldError::BlobHashInvalid { .. } => {
            StorageChallengeFailureReason::HashMismatch
        }
        WorldError::Io(_) => StorageChallengeFailureReason::ReadIoError,
        WorldError::DistributedValidationFailed { reason } => {
            let lower = reason.to_ascii_lowercase();
            if lower.contains("timeout") || lower.contains("window") {
                StorageChallengeFailureReason::Timeout
            } else if lower.contains("hash") {
                StorageChallengeFailureReason::HashMismatch
            } else if lower.contains("sample") || lower.contains("missing") {
                StorageChallengeFailureReason::MissingSample
            } else if lower.contains("signature") || lower.contains("proof") {
                StorageChallengeFailureReason::SignatureInvalid
            } else {
                StorageChallengeFailureReason::Unknown
            }
        }
        WorldError::SignatureKeyInvalid => StorageChallengeFailureReason::SignatureInvalid,
        WorldError::NetworkProtocolUnavailable { .. }
        | WorldError::NetworkRequestFailed { .. }
        | WorldError::Serde(_) => StorageChallengeFailureReason::Unknown,
    }
}

fn classify_receipt_failure_reason(
    challenge: &StorageChallenge,
    receipt: &StorageChallengeReceipt,
    allowed_clock_skew_ms: i64,
) -> StorageChallengeFailureReason {
    if let Some(reason) = receipt.failure_reason {
        return reason;
    }
    if receipt.proof_kind != STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1 {
        return StorageChallengeFailureReason::SignatureInvalid;
    }
    if challenge.sample_offset != receipt.sample_offset
        || challenge.sample_size_bytes != receipt.sample_size_bytes
    {
        return StorageChallengeFailureReason::MissingSample;
    }
    if challenge.content_hash != receipt.content_hash
        || challenge.expected_sample_hash != receipt.sample_hash
    {
        return StorageChallengeFailureReason::HashMismatch;
    }
    let min_time = challenge
        .issued_at_unix_ms
        .saturating_sub(allowed_clock_skew_ms);
    let max_time = challenge
        .expires_at_unix_ms
        .saturating_add(allowed_clock_skew_ms);
    if receipt.responded_at_unix_ms < min_time || receipt.responded_at_unix_ms > max_time {
        return StorageChallengeFailureReason::Timeout;
    }
    StorageChallengeFailureReason::Unknown
}

fn signing_key_from_hex(private_key_hex: &str) -> Result<SigningKey, String> {
    let private_bytes = decode_hex_array::<32>(private_key_hex, "distfs signer private key")?;
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
    use agent_world::runtime::BlobStore;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-distfs-net-{prefix}-{unique}"))
    }

    fn deterministic_keypair_hex(seed: u8) -> (String, String) {
        let bytes = [seed; 32];
        let signing_key = SigningKey::from_bytes(&bytes);
        (
            hex::encode(signing_key.to_bytes()),
            hex::encode(signing_key.verifying_key().to_bytes()),
        )
    }

    #[derive(Clone, Default)]
    struct TestInMemoryNetwork {
        inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    }

    impl DistributedNetwork<WorldError> for TestInMemoryNetwork {
        fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
            let mut inbox = self.inbox.lock().expect("lock inbox");
            inbox
                .entry(topic.to_string())
                .or_default()
                .push(payload.to_vec());
            Ok(())
        }

        fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
            let mut inbox = self.inbox.lock().expect("lock inbox");
            inbox.entry(topic.to_string()).or_default();
            Ok(NetworkSubscription::new(
                topic.to_string(),
                Arc::clone(&self.inbox),
            ))
        }

        fn request(&self, _protocol: &str, _payload: &[u8]) -> Result<Vec<u8>, WorldError> {
            Err(WorldError::NetworkProtocolUnavailable {
                protocol: "test".to_string(),
            })
        }

        fn register_handler(
            &self,
            _protocol: &str,
            _handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
        ) -> Result<(), WorldError> {
            Ok(())
        }
    }

    fn sample_request_envelope() -> DistfsChallengeRequestEnvelope {
        let store_dir = temp_dir("request-envelope");
        let store = LocalCasStore::new(&store_dir);
        let content_hash = store
            .put_bytes(b"network proof payload")
            .expect("put content hash");
        let challenge = store
            .issue_storage_challenge(&StorageChallengeRequest {
                challenge_id: "challenge-1".to_string(),
                world_id: "w1".to_string(),
                node_id: "node-b".to_string(),
                content_hash,
                max_sample_bytes: 32,
                issued_at_unix_ms: 1_000,
                challenge_ttl_ms: 30_000,
                vrf_seed: "seed-1".to_string(),
            })
            .expect("issue challenge");
        let (private_key_hex, public_key_hex) = deterministic_keypair_hex(7);
        let envelope = sign_distfs_challenge_request(
            "w1",
            "node-a",
            private_key_hex.as_str(),
            public_key_hex.as_str(),
            "node-b",
            challenge,
            1_001,
        )
        .expect("sign request");
        let _ = std::fs::remove_dir_all(store_dir);
        envelope
    }

    #[test]
    fn distfs_challenge_topics_use_expected_suffixes() {
        assert_eq!(
            distfs_challenge_request_topic("w1"),
            "aw.w1.distfs.challenge.request"
        );
        assert_eq!(distfs_challenge_proof_topic("w1"), "aw.w1.distfs.challenge.proof");
    }

    #[test]
    fn distfs_challenge_request_signature_verifies() {
        let request = sample_request_envelope();
        verify_distfs_challenge_request(&request).expect("verify request");
    }

    #[test]
    fn distfs_challenge_request_signature_rejects_tamper() {
        let mut request = sample_request_envelope();
        request.target_node_id = "node-c".to_string();
        let err = verify_distfs_challenge_request(&request).expect_err("tamper should fail");
        assert!(err.contains("target_node_id mismatch") || err.contains("signature failed"));
    }

    #[test]
    fn distfs_network_driver_roundtrip_challenge_and_proof() {
        let storage_a = temp_dir("storage-a");
        let storage_b = temp_dir("storage-b");
        let store_a = LocalCasStore::new(&storage_a);
        let store_b = LocalCasStore::new(&storage_b);
        let blob = b"network-proof-data";
        let hash_a = store_a.put_bytes(blob).expect("put a");
        let hash_b = store_b.put_bytes(blob).expect("put b");
        assert_eq!(hash_a, hash_b);

        let network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync> =
            Arc::new(TestInMemoryNetwork::default());
        let (private_a, public_a) = deterministic_keypair_hex(31);
        let (private_b, public_b) = deterministic_keypair_hex(32);
        let mut driver_a = DistfsChallengeNetworkDriver::new(
            "w1",
            "node-a",
            private_a.as_str(),
            public_a.as_str(),
            storage_a.clone(),
            DistfsProbeRuntimeConfig {
                max_sample_bytes: 64,
                challenges_per_tick: 1,
                challenge_ttl_ms: 30_000,
                allowed_clock_skew_ms: 5_000,
                adaptive_policy: agent_world_distfs::StorageChallengeAdaptivePolicy::default(),
            },
            Arc::clone(&network),
        )
        .expect("driver a");
        let mut driver_b = DistfsChallengeNetworkDriver::new(
            "w1",
            "node-b",
            private_b.as_str(),
            public_b.as_str(),
            storage_b.clone(),
            DistfsProbeRuntimeConfig {
                max_sample_bytes: 64,
                challenges_per_tick: 1,
                challenge_ttl_ms: 30_000,
                allowed_clock_skew_ms: 5_000,
                adaptive_policy: agent_world_distfs::StorageChallengeAdaptivePolicy::default(),
            },
            Arc::clone(&network),
        )
        .expect("driver b");

        driver_a.register_observation_role("node-b", NodeRole::Storage);

        let tick_a_1 = driver_a.tick(1_000);
        assert_eq!(tick_a_1.mode, "network");
        assert_eq!(tick_a_1.issued_challenge_ids.len(), 1);

        let tick_b_1 = driver_b.tick(1_001);
        assert_eq!(tick_b_1.mode, "fallback_local");
        assert_eq!(tick_b_1.answered_challenge_ids.len(), 1);

        let tick_a_2 = driver_a.tick(1_002);
        assert_eq!(tick_a_2.mode, "network");
        let report = tick_a_2.probe_report.expect("probe report");
        assert!(report.total_checks >= 1);
        assert!(report.passed_checks >= 1);
        assert!(tick_a_2.accepted_proof_ids.len() >= 1);

        let _ = std::fs::remove_dir_all(storage_a);
        let _ = std::fs::remove_dir_all(storage_b);
    }
}
