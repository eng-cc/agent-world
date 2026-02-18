use std::fs;
use std::path::{Path, PathBuf};

use agent_world_distfs::{
    apply_replication_record, build_replication_record, FileReplicationRecord, LocalCasStore,
    SingleWriterReplicationGuard,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{NodeError, PosConsensusStatus, PosDecision};

const REPLICATION_VERSION: u8 = 1;
const COMMIT_FILE_PREFIX: &str = "consensus/commits";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeReplicationConfig {
    pub root_dir: PathBuf,
    signing_private_key_hex: Option<String>,
    signing_public_key_hex: Option<String>,
    enforce_signature: bool,
}

impl NodeReplicationConfig {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, NodeError> {
        let root_dir = root_dir.into();
        if root_dir.as_os_str().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "replication root_dir cannot be empty".to_string(),
            });
        }
        Ok(Self {
            root_dir,
            signing_private_key_hex: None,
            signing_public_key_hex: None,
            enforce_signature: false,
        })
    }

    pub fn with_signing_keypair(
        mut self,
        private_key_hex: impl Into<String>,
        public_key_hex: impl Into<String>,
    ) -> Result<Self, NodeError> {
        let private_key_hex = private_key_hex.into();
        let public_key_hex = public_key_hex.into();
        let signing_key = signing_key_from_hex(private_key_hex.as_str())?;
        let expected_public = hex::encode(signing_key.verifying_key().to_bytes());
        if expected_public != public_key_hex {
            return Err(NodeError::InvalidConfig {
                reason: "replication signing public key does not match private key".to_string(),
            });
        }
        self.signing_private_key_hex = Some(private_key_hex);
        self.signing_public_key_hex = Some(public_key_hex);
        self.enforce_signature = true;
        Ok(self)
    }

    fn signing_keypair(&self) -> Result<Option<ReplicationSigningKey>, NodeError> {
        match (
            self.signing_private_key_hex.as_deref(),
            self.signing_public_key_hex.as_deref(),
        ) {
            (Some(private_key_hex), Some(public_key_hex)) => {
                let signing_key = signing_key_from_hex(private_key_hex)?;
                let expected_public = hex::encode(signing_key.verifying_key().to_bytes());
                if expected_public != public_key_hex {
                    return Err(NodeError::InvalidConfig {
                        reason: "replication signing public key does not match private key"
                            .to_string(),
                    });
                }
                Ok(Some(ReplicationSigningKey {
                    signing_key,
                    public_key_hex: public_key_hex.to_string(),
                }))
            }
            (None, None) => Ok(None),
            _ => Err(NodeError::InvalidConfig {
                reason: "replication signing keypair must include both private/public".to_string(),
            }),
        }
    }

    pub(crate) fn consensus_signer(&self) -> Result<Option<(SigningKey, String)>, NodeError> {
        Ok(self
            .signing_keypair()?
            .map(|key| (key.signing_key, key.public_key_hex)))
    }

    pub(crate) fn enforce_consensus_signature(&self) -> bool {
        self.enforce_signature || self.signing_private_key_hex.is_some()
    }

    fn store_root(&self) -> PathBuf {
        self.root_dir.join("store")
    }

    fn guard_state_path(&self) -> PathBuf {
        self.root_dir.join("replication_guard.json")
    }

    fn writer_state_path(&self, node_id: &str) -> PathBuf {
        self.root_dir
            .join(format!("replication_writer_state_{node_id}.json"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GossipReplicationMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    pub record: FileReplicationRecord,
    pub payload: Vec<u8>,
    pub public_key_hex: Option<String>,
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone)]
struct ReplicationSigningKey {
    signing_key: SigningKey,
    public_key_hex: String,
}

#[derive(Debug)]
pub(crate) struct ReplicationRuntime {
    config: NodeReplicationConfig,
    store: LocalCasStore,
    guard: SingleWriterReplicationGuard,
    writer_state: LocalWriterState,
    signer: Option<ReplicationSigningKey>,
    enforce_signature: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct LocalWriterState {
    last_sequence: u64,
    last_replicated_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReplicatedCommitPayload {
    world_id: String,
    node_id: String,
    height: u64,
    slot: u64,
    epoch: u64,
    block_hash: String,
    committed_at_ms: i64,
    execution_block_hash: Option<String>,
    execution_state_root: Option<String>,
}

#[derive(Debug, Serialize)]
struct ReplicationSigningPayload<'a> {
    version: u8,
    world_id: &'a str,
    node_id: &'a str,
    record: &'a FileReplicationRecord,
    payload: &'a [u8],
    public_key_hex: Option<&'a str>,
}

impl ReplicationRuntime {
    pub(crate) fn new(config: &NodeReplicationConfig, node_id: &str) -> Result<Self, NodeError> {
        fs::create_dir_all(&config.root_dir).map_err(|err| NodeError::Replication {
            reason: format!(
                "create replication root {} failed: {}",
                config.root_dir.display(),
                err
            ),
        })?;

        let guard = load_json_or_default::<SingleWriterReplicationGuard>(
            config.guard_state_path().as_path(),
        )?;
        let writer_state =
            load_json_or_default::<LocalWriterState>(config.writer_state_path(node_id).as_path())?;
        let signer = config.signing_keypair()?;

        Ok(Self {
            config: config.clone(),
            store: LocalCasStore::new(config.store_root()),
            guard,
            writer_state,
            enforce_signature: config.enforce_signature || signer.is_some(),
            signer,
        })
    }

    pub(crate) fn build_local_commit_message(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
        decision: &PosDecision,
        execution_block_hash: Option<&str>,
        execution_state_root: Option<&str>,
    ) -> Result<Option<GossipReplicationMessage>, NodeError> {
        if !matches!(decision.status, PosConsensusStatus::Committed) {
            return Ok(None);
        }
        if decision.height <= self.writer_state.last_replicated_height {
            return Ok(None);
        }
        if execution_block_hash.is_some() != execution_state_root.is_some() {
            return Err(NodeError::Replication {
                reason: "replication execution hash binding requires both block/state".to_string(),
            });
        }

        let payload = ReplicatedCommitPayload {
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            height: decision.height,
            slot: decision.slot,
            epoch: decision.epoch,
            block_hash: decision.block_hash.clone(),
            committed_at_ms: now_ms,
            execution_block_hash: execution_block_hash.map(str::to_string),
            execution_state_root: execution_state_root.map(str::to_string),
        };
        let payload_bytes = serde_json::to_vec(&payload).map_err(|err| NodeError::Replication {
            reason: format!("serialize local replication payload failed: {}", err),
        })?;
        let writer_id = self
            .signer
            .as_ref()
            .map(|signer| signer.public_key_hex.as_str())
            .unwrap_or(node_id);
        let sequence = self
            .guard
            .last_sequence
            .max(self.writer_state.last_sequence)
            .saturating_add(1);
        let path = format!("{COMMIT_FILE_PREFIX}/{:020}.json", decision.height);
        let record = build_replication_record(
            world_id,
            writer_id,
            sequence,
            path.as_str(),
            &payload_bytes,
            now_ms,
        )
        .map_err(distfs_error_to_node_error)?;

        apply_replication_record(&self.store, &mut self.guard, &record, &payload_bytes)
            .map_err(distfs_error_to_node_error)?;

        self.writer_state.last_sequence = record.sequence;
        self.writer_state.last_replicated_height = decision.height;
        self.persist_state(node_id)?;

        let mut message = GossipReplicationMessage {
            version: REPLICATION_VERSION,
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            record,
            payload: payload_bytes,
            public_key_hex: self
                .signer
                .as_ref()
                .map(|signer| signer.public_key_hex.clone()),
            signature_hex: None,
        };

        if let Some(signer) = &self.signer {
            let signature_hex = sign_replication_message(&message, signer)?;
            message.signature_hex = Some(signature_hex);
        }

        Ok(Some(message))
    }

    pub(crate) fn apply_remote_message(
        &mut self,
        node_id: &str,
        world_id: &str,
        message: &GossipReplicationMessage,
    ) -> Result<(), NodeError> {
        if message.version != REPLICATION_VERSION {
            return Ok(());
        }
        if message.node_id == node_id {
            return Ok(());
        }
        if message.world_id != world_id || message.record.world_id != world_id {
            return Ok(());
        }

        if self.enforce_signature
            || message.signature_hex.is_some()
            || message.public_key_hex.is_some()
        {
            verify_replication_message_signature(message)?;
            if let Some(public_key_hex) = message.public_key_hex.as_deref() {
                if message.record.writer_id != public_key_hex {
                    return Err(NodeError::Replication {
                        reason: "replication writer_id does not match signature public key"
                            .to_string(),
                    });
                }
            }
        }

        if let Some(existing_writer) = self.guard.writer_id.as_deref() {
            if existing_writer != message.record.writer_id.as_str() {
                return Err(NodeError::Replication {
                    reason: format!(
                        "replication writer conflict: expected={}, got={}",
                        existing_writer, message.record.writer_id
                    ),
                });
            }
        }
        if message.record.sequence <= self.guard.last_sequence {
            return Ok(());
        }

        apply_replication_record(
            &self.store,
            &mut self.guard,
            &message.record,
            &message.payload,
        )
        .map_err(distfs_error_to_node_error)?;

        write_json_pretty(self.config.guard_state_path().as_path(), &self.guard)
    }

    fn persist_state(&self, node_id: &str) -> Result<(), NodeError> {
        write_json_pretty(self.config.guard_state_path().as_path(), &self.guard)?;
        write_json_pretty(
            self.config.writer_state_path(node_id).as_path(),
            &self.writer_state,
        )
    }
}

fn signing_key_from_hex(private_key_hex: &str) -> Result<SigningKey, NodeError> {
    let private_key = decode_hex_array::<32>(private_key_hex, "replication private key")?;
    Ok(SigningKey::from_bytes(&private_key))
}

fn sign_replication_message(
    message: &GossipReplicationMessage,
    signer: &ReplicationSigningKey,
) -> Result<String, NodeError> {
    let payload = replication_signing_bytes(message)?;
    let signature: Signature = signer.signing_key.sign(&payload);
    Ok(hex::encode(signature.to_bytes()))
}

fn verify_replication_message_signature(
    message: &GossipReplicationMessage,
) -> Result<(), NodeError> {
    let public_key_hex =
        message
            .public_key_hex
            .as_deref()
            .ok_or_else(|| NodeError::Replication {
                reason: "replication signature missing public_key_hex".to_string(),
            })?;
    let signature_hex = message
        .signature_hex
        .as_deref()
        .ok_or_else(|| NodeError::Replication {
            reason: "replication signature missing signature_hex".to_string(),
        })?;

    let public_key_bytes = decode_hex_array::<32>(public_key_hex, "replication public key")?;
    let signature_bytes = decode_hex_array::<64>(signature_hex, "replication signature")?;
    let public_key =
        VerifyingKey::from_bytes(&public_key_bytes).map_err(|err| NodeError::Replication {
            reason: format!("parse replication public key failed: {}", err),
        })?;
    let signature = Signature::from_bytes(&signature_bytes);
    let payload = replication_signing_bytes(message)?;
    public_key
        .verify(&payload, &signature)
        .map_err(|err| NodeError::Replication {
            reason: format!("verify replication signature failed: {}", err),
        })
}

fn replication_signing_bytes(message: &GossipReplicationMessage) -> Result<Vec<u8>, NodeError> {
    let payload = ReplicationSigningPayload {
        version: message.version,
        world_id: message.world_id.as_str(),
        node_id: message.node_id.as_str(),
        record: &message.record,
        payload: &message.payload,
        public_key_hex: message.public_key_hex.as_deref(),
    };
    serde_json::to_vec(&payload).map_err(|err| NodeError::Replication {
        reason: format!("serialize replication signing payload failed: {}", err),
    })
}

fn decode_hex_array<const N: usize>(value: &str, label: &str) -> Result<[u8; N], NodeError> {
    let bytes = hex::decode(value).map_err(|_| NodeError::Replication {
        reason: format!("{} must be valid hex", label),
    })?;
    bytes.try_into().map_err(|_| NodeError::Replication {
        reason: format!("{} must be {} bytes hex", label, N),
    })
}

fn load_json_or_default<T>(path: &Path) -> Result<T, NodeError>
where
    T: for<'de> Deserialize<'de> + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let bytes = fs::read(path).map_err(|err| NodeError::Replication {
        reason: format!("read {} failed: {}", path.display(), err),
    })?;
    serde_json::from_slice::<T>(&bytes).map_err(|err| NodeError::Replication {
        reason: format!("parse {} failed: {}", path.display(), err),
    })
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), NodeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| NodeError::Replication {
            reason: format!("create dir {} failed: {}", parent.display(), err),
        })?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| NodeError::Replication {
        reason: format!("serialize {} failed: {}", path.display(), err),
    })?;
    fs::write(path, bytes).map_err(|err| NodeError::Replication {
        reason: format!("write {} failed: {}", path.display(), err),
    })
}

fn distfs_error_to_node_error<E>(err: E) -> NodeError
where
    E: std::fmt::Debug,
{
    NodeError::Replication {
        reason: format!("{err:?}"),
    }
}
