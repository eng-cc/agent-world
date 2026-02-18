use std::fs;
use std::path::Path;

#[cfg(test)]
use agent_world::consensus_action_payload::ConsensusActionPayloadEnvelope;
use agent_world::consensus_action_payload::{
    decode_consensus_action_payload, ConsensusActionPayloadBody,
};
use agent_world::runtime::{blake3_hex, BlobStore, LocalCasStore, World as RuntimeWorld};
use agent_world_node::{
    compute_consensus_action_root, NodeExecutionCommitContext, NodeExecutionCommitResult,
    NodeExecutionHook, NodeSnapshot,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionBridgeState {
    pub last_applied_committed_height: u64,
    pub last_execution_block_hash: Option<String>,
    pub last_execution_state_root: Option<String>,
    pub last_node_block_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionBridgeRecord {
    pub world_id: String,
    pub height: u64,
    pub node_block_hash: Option<String>,
    pub execution_block_hash: String,
    pub execution_state_root: String,
    pub journal_len: usize,
    pub snapshot_ref: String,
    pub journal_ref: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ExecutionHashPayload<'a> {
    world_id: &'a str,
    height: u64,
    prev_execution_block_hash: &'a str,
    execution_state_root: &'a str,
    journal_len: usize,
}

pub(super) struct NodeRuntimeExecutionDriver {
    state_path: std::path::PathBuf,
    world_dir: std::path::PathBuf,
    records_dir: std::path::PathBuf,
    execution_store: LocalCasStore,
    state: ExecutionBridgeState,
    execution_world: RuntimeWorld,
}

impl NodeRuntimeExecutionDriver {
    pub(super) fn new(
        state_path: std::path::PathBuf,
        world_dir: std::path::PathBuf,
        records_dir: std::path::PathBuf,
        storage_root: std::path::PathBuf,
    ) -> Result<Self, String> {
        let state = load_execution_bridge_state(state_path.as_path())?;
        let execution_world = load_execution_world(world_dir.as_path())?;
        Ok(Self {
            state_path,
            world_dir,
            records_dir,
            execution_store: LocalCasStore::new(storage_root),
            state,
            execution_world,
        })
    }
}

impl NodeExecutionHook for NodeRuntimeExecutionDriver {
    fn on_commit(
        &mut self,
        context: NodeExecutionCommitContext,
    ) -> Result<NodeExecutionCommitResult, String> {
        if context.height < self.state.last_applied_committed_height {
            return Err(format!(
                "execution driver received stale height: context={} state={}",
                context.height, self.state.last_applied_committed_height
            ));
        }
        if context.height == self.state.last_applied_committed_height {
            let execution_block_hash =
                self.state
                    .last_execution_block_hash
                    .clone()
                    .ok_or_else(|| {
                        "execution driver missing block hash for current height".to_string()
                    })?;
            let execution_state_root =
                self.state
                    .last_execution_state_root
                    .clone()
                    .ok_or_else(|| {
                        "execution driver missing state root for current height".to_string()
                    })?;
            return Ok(NodeExecutionCommitResult {
                execution_height: context.height,
                execution_block_hash,
                execution_state_root,
            });
        }
        if context.height != self.state.last_applied_committed_height.saturating_add(1) {
            return Err(format!(
                "execution driver requires contiguous committed heights: last_applied={} incoming={}",
                self.state.last_applied_committed_height, context.height
            ));
        }

        let computed_action_root =
            compute_consensus_action_root(context.committed_actions.as_slice())
                .map_err(|err| format!("execution driver compute action root failed: {err:?}"))?;
        if computed_action_root != context.action_root {
            return Err(format!(
                "execution driver action_root mismatch expected={} actual={}",
                computed_action_root, context.action_root
            ));
        }

        let mut decoded_actions = Vec::with_capacity(context.committed_actions.len());
        for action in &context.committed_actions {
            match decode_consensus_action_payload(action.payload_cbor.as_slice()) {
                Ok(ConsensusActionPayloadBody::RuntimeAction { action: decoded }) => {
                    decoded_actions.push(decoded);
                }
                Ok(ConsensusActionPayloadBody::SimulatorAction { .. }) => {}
                Err(err) => {
                    return Err(format!(
                        "execution driver decode committed action failed action_id={} err={}",
                        action.action_id, err
                    ));
                }
            }
        }

        fs::create_dir_all(self.records_dir.as_path()).map_err(|err| {
            format!(
                "create execution records dir {} failed: {}",
                self.records_dir.display(),
                err
            )
        })?;

        for action in decoded_actions {
            self.execution_world.submit_action(action);
        }
        self.execution_world.step().map_err(|err| {
            format!(
                "execution driver world.step failed at height {}: {:?}",
                context.height, err
            )
        })?;

        let snapshot_value = self.execution_world.snapshot();
        let journal_value = self.execution_world.journal().clone();
        let snapshot_bytes = to_cbor(snapshot_value)?;
        let journal_bytes = to_cbor(journal_value)?;

        let snapshot_ref = self
            .execution_store
            .put_bytes(snapshot_bytes.as_slice())
            .map_err(|err| format!("execution driver CAS snapshot put failed: {:?}", err))?;
        let journal_ref = self
            .execution_store
            .put_bytes(journal_bytes.as_slice())
            .map_err(|err| format!("execution driver CAS journal put failed: {:?}", err))?;

        let execution_state_root = blake3_hex(snapshot_bytes.as_slice());
        let prev_execution_block_hash = self
            .state
            .last_execution_block_hash
            .clone()
            .unwrap_or_else(|| "genesis".to_string());
        let hash_payload = ExecutionHashPayload {
            world_id: context.world_id.as_str(),
            height: context.height,
            prev_execution_block_hash: prev_execution_block_hash.as_str(),
            execution_state_root: execution_state_root.as_str(),
            journal_len: self.execution_world.journal().len(),
        };
        let execution_block_hash = blake3_hex(to_cbor(hash_payload)?.as_slice());
        let node_block_hash = Some(context.node_block_hash.clone());

        let record = ExecutionBridgeRecord {
            world_id: context.world_id.clone(),
            height: context.height,
            node_block_hash: node_block_hash.clone(),
            execution_block_hash: execution_block_hash.clone(),
            execution_state_root: execution_state_root.clone(),
            journal_len: self.execution_world.journal().len(),
            snapshot_ref,
            journal_ref,
            timestamp_ms: context.committed_at_unix_ms,
        };
        persist_execution_bridge_record(self.records_dir.as_path(), &record)?;

        self.state.last_applied_committed_height = context.height;
        self.state.last_execution_block_hash = Some(execution_block_hash);
        self.state.last_execution_state_root = Some(execution_state_root);
        self.state.last_node_block_hash = node_block_hash;

        persist_execution_bridge_state(self.state_path.as_path(), &self.state)?;
        persist_execution_world(self.world_dir.as_path(), &self.execution_world)?;

        Ok(NodeExecutionCommitResult {
            execution_height: context.height,
            execution_block_hash: self
                .state
                .last_execution_block_hash
                .clone()
                .ok_or_else(|| "execution driver missing execution_block_hash".to_string())?,
            execution_state_root: self
                .state
                .last_execution_state_root
                .clone()
                .ok_or_else(|| "execution driver missing execution_state_root".to_string())?,
        })
    }
}

pub(super) fn load_execution_bridge_state(path: &Path) -> Result<ExecutionBridgeState, String> {
    if !path.exists() {
        return Ok(ExecutionBridgeState::default());
    }
    let bytes = fs::read(path).map_err(|err| {
        format!(
            "read execution bridge state {} failed: {}",
            path.display(),
            err
        )
    })?;
    serde_json::from_slice::<ExecutionBridgeState>(bytes.as_slice()).map_err(|err| {
        format!(
            "parse execution bridge state {} failed: {}",
            path.display(),
            err
        )
    })
}

pub(super) fn persist_execution_bridge_state(
    path: &Path,
    state: &ExecutionBridgeState,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|err| format!("serialize execution bridge state failed: {}", err))?;
    super::write_bytes_atomic(path, bytes.as_slice())
}

pub(super) fn load_execution_world(world_dir: &Path) -> Result<RuntimeWorld, String> {
    let snapshot_path = world_dir.join("snapshot.json");
    let journal_path = world_dir.join("journal.json");
    if !snapshot_path.exists() || !journal_path.exists() {
        return Ok(RuntimeWorld::new());
    }
    RuntimeWorld::load_from_dir(world_dir).map_err(|err| {
        format!(
            "load execution world from {} failed: {:?}",
            world_dir.display(),
            err
        )
    })
}

pub(super) fn persist_execution_world(
    world_dir: &Path,
    execution_world: &RuntimeWorld,
) -> Result<(), String> {
    execution_world.save_to_dir(world_dir).map_err(|err| {
        format!(
            "save execution world to {} failed: {:?}",
            world_dir.display(),
            err
        )
    })
}

pub(super) fn bridge_committed_heights(
    snapshot: &NodeSnapshot,
    observed_at_unix_ms: i64,
    execution_world: &mut RuntimeWorld,
    execution_store: &LocalCasStore,
    execution_records_dir: &Path,
    state: &mut ExecutionBridgeState,
) -> Result<Vec<ExecutionBridgeRecord>, String> {
    let target_height = snapshot.consensus.committed_height;
    if target_height <= state.last_applied_committed_height {
        return Ok(Vec::new());
    }

    fs::create_dir_all(execution_records_dir).map_err(|err| {
        format!(
            "create execution records dir {} failed: {}",
            execution_records_dir.display(),
            err
        )
    })?;

    let mut records = Vec::new();
    for height in (state.last_applied_committed_height + 1)..=target_height {
        execution_world.step().map_err(|err| {
            format!(
                "execution bridge world.step failed at height {}: {:?}",
                height, err
            )
        })?;

        let snapshot_value = execution_world.snapshot();
        let journal_value = execution_world.journal().clone();
        let snapshot_bytes = to_cbor(snapshot_value)?;
        let journal_bytes = to_cbor(journal_value)?;

        let snapshot_ref = execution_store
            .put_bytes(snapshot_bytes.as_slice())
            .map_err(|err| format!("execution bridge CAS snapshot put failed: {:?}", err))?;
        let journal_ref = execution_store
            .put_bytes(journal_bytes.as_slice())
            .map_err(|err| format!("execution bridge CAS journal put failed: {:?}", err))?;

        let execution_state_root = blake3_hex(snapshot_bytes.as_slice());
        let prev_execution_block_hash = state
            .last_execution_block_hash
            .clone()
            .unwrap_or_else(|| "genesis".to_string());
        let hash_payload = ExecutionHashPayload {
            world_id: snapshot.world_id.as_str(),
            height,
            prev_execution_block_hash: prev_execution_block_hash.as_str(),
            execution_state_root: execution_state_root.as_str(),
            journal_len: execution_world.journal().len(),
        };
        let execution_block_hash = blake3_hex(to_cbor(hash_payload)?.as_slice());
        let node_block_hash = if height == target_height {
            snapshot.consensus.last_block_hash.clone()
        } else {
            None
        };

        let record = ExecutionBridgeRecord {
            world_id: snapshot.world_id.clone(),
            height,
            node_block_hash: node_block_hash.clone(),
            execution_block_hash: execution_block_hash.clone(),
            execution_state_root: execution_state_root.clone(),
            journal_len: execution_world.journal().len(),
            snapshot_ref,
            journal_ref,
            timestamp_ms: observed_at_unix_ms,
        };
        persist_execution_bridge_record(execution_records_dir, &record)?;

        state.last_applied_committed_height = height;
        state.last_execution_block_hash = Some(execution_block_hash);
        state.last_execution_state_root = Some(execution_state_root);
        state.last_node_block_hash = node_block_hash;
        records.push(record);
    }

    Ok(records)
}

fn persist_execution_bridge_record(
    execution_records_dir: &Path,
    record: &ExecutionBridgeRecord,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(record)
        .map_err(|err| format!("serialize execution bridge record failed: {}", err))?;
    let path = execution_records_dir.join(format!("{:020}.json", record.height));
    super::write_bytes_atomic(path.as_path(), bytes.as_slice())?;

    let latest_path = execution_records_dir.join("latest.json");
    super::write_bytes_atomic(latest_path.as_path(), bytes.as_slice())
}

fn to_cbor<T: Serialize>(value: T) -> Result<Vec<u8>, String> {
    serde_cbor::to_vec(&value).map_err(|err| format!("serialize to cbor failed: {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::consensus_action_payload::encode_consensus_action_payload;
    use agent_world::simulator::{Action as SimulatorAction, ActionSubmitter};
    use agent_world_node::{NodeConsensusSnapshot, NodeRole};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
    }

    fn sample_snapshot(committed_height: u64, block_hash: Option<&str>) -> NodeSnapshot {
        NodeSnapshot {
            node_id: "node-a".to_string(),
            player_id: "node-a".to_string(),
            world_id: "w1".to_string(),
            role: NodeRole::Sequencer,
            running: true,
            tick_count: 10,
            last_tick_unix_ms: Some(10),
            consensus: NodeConsensusSnapshot {
                committed_height,
                last_block_hash: block_hash.map(ToOwned::to_owned),
                ..NodeConsensusSnapshot::default()
            },
            last_error: None,
        }
    }

    #[test]
    fn bridge_committed_heights_persists_records_and_state() {
        let dir = temp_dir("execution-bridge");
        let store = LocalCasStore::new(dir.join("store"));
        let mut world = RuntimeWorld::new();
        let mut state = ExecutionBridgeState::default();
        let records_dir = dir.join("records");

        let snapshot = sample_snapshot(2, Some("node-h2"));
        let records = bridge_committed_heights(
            &snapshot,
            1_000,
            &mut world,
            &store,
            records_dir.as_path(),
            &mut state,
        )
        .expect("bridge");

        assert_eq!(records.len(), 2);
        assert_eq!(state.last_applied_committed_height, 2);
        assert_eq!(state.last_node_block_hash.as_deref(), Some("node-h2"));
        assert!(records_dir.join("00000000000000000001.json").exists());
        assert!(records_dir.join("00000000000000000002.json").exists());
        assert!(records_dir.join("latest.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bridge_committed_heights_is_noop_when_height_not_advanced() {
        let dir = temp_dir("execution-bridge-noop");
        let store = LocalCasStore::new(dir.join("store"));
        let mut world = RuntimeWorld::new();
        let mut state = ExecutionBridgeState {
            last_applied_committed_height: 3,
            last_execution_block_hash: Some("h3".to_string()),
            last_execution_state_root: Some("s3".to_string()),
            last_node_block_hash: Some("node-h3".to_string()),
        };

        let snapshot = sample_snapshot(3, Some("node-h3"));
        let records = bridge_committed_heights(
            &snapshot,
            1_100,
            &mut world,
            &store,
            dir.join("records").as_path(),
            &mut state,
        )
        .expect("bridge");

        assert!(records.is_empty());
        assert_eq!(state.last_applied_committed_height, 3);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_bridge_state_roundtrip() {
        let dir = temp_dir("execution-bridge-state");
        let state_path = dir.join("state.json");
        let state = ExecutionBridgeState {
            last_applied_committed_height: 9,
            last_execution_block_hash: Some("exec-h9".to_string()),
            last_execution_state_root: Some("exec-s9".to_string()),
            last_node_block_hash: Some("node-h9".to_string()),
        };

        persist_execution_bridge_state(state_path.as_path(), &state).expect("persist");
        let loaded = load_execution_bridge_state(state_path.as_path()).expect("load");
        assert_eq!(loaded, state);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_world_persistence_roundtrip() {
        let dir = temp_dir("execution-world");
        let world_dir = dir.join("world");
        let world = RuntimeWorld::new();

        persist_execution_world(world_dir.as_path(), &world).expect("persist world");
        let loaded = load_execution_world(world_dir.as_path()).expect("load world");
        assert_eq!(loaded.journal().len(), world.journal().len());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn node_runtime_execution_driver_persists_chain_records() {
        let dir = temp_dir("execution-driver");
        let state_path = dir.join("state.json");
        let world_dir = dir.join("world");
        let records_dir = dir.join("records");
        let storage_root = dir.join("store");
        let mut driver = NodeRuntimeExecutionDriver::new(
            state_path.clone(),
            world_dir.clone(),
            records_dir.clone(),
            storage_root,
        )
        .expect("driver");
        let empty_action_root = compute_consensus_action_root(&[]).expect("empty action root");

        let first = driver
            .on_commit(NodeExecutionCommitContext {
                world_id: "w1".to_string(),
                node_id: "node-a".to_string(),
                height: 1,
                slot: 0,
                epoch: 0,
                node_block_hash: "node-h1".to_string(),
                action_root: empty_action_root.clone(),
                committed_actions: Vec::new(),
                committed_at_unix_ms: 1_000,
            })
            .expect("first commit");
        let second = driver
            .on_commit(NodeExecutionCommitContext {
                world_id: "w1".to_string(),
                node_id: "node-a".to_string(),
                height: 2,
                slot: 1,
                epoch: 0,
                node_block_hash: "node-h2".to_string(),
                action_root: empty_action_root,
                committed_actions: Vec::new(),
                committed_at_unix_ms: 2_000,
            })
            .expect("second commit");

        assert_eq!(first.execution_height, 1);
        assert_eq!(second.execution_height, 2);
        assert_ne!(first.execution_block_hash, second.execution_block_hash);
        assert!(records_dir.join("00000000000000000001.json").exists());
        assert!(records_dir.join("00000000000000000002.json").exists());

        let state = load_execution_bridge_state(state_path.as_path()).expect("load state");
        assert_eq!(state.last_applied_committed_height, 2);
        assert_eq!(state.last_node_block_hash.as_deref(), Some("node-h2"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn node_runtime_execution_driver_ignores_simulator_payload_envelope() {
        let dir = temp_dir("execution-driver-simulator-payload");
        let state_path = dir.join("state.json");
        let world_dir = dir.join("world");
        let records_dir = dir.join("records");
        let storage_root = dir.join("store");
        let mut driver = NodeRuntimeExecutionDriver::new(
            state_path.clone(),
            world_dir,
            records_dir.clone(),
            storage_root,
        )
        .expect("driver");

        let payload = encode_consensus_action_payload(
            &ConsensusActionPayloadEnvelope::from_simulator_action(
                SimulatorAction::HarvestRadiation {
                    agent_id: "agent-0".to_string(),
                    max_amount: 1,
                },
                ActionSubmitter::System,
            ),
        )
        .expect("encode simulator payload");
        let committed_action =
            agent_world_node::NodeConsensusAction::from_payload(1, "node-a", payload)
                .expect("consensus action");
        let action_root =
            compute_consensus_action_root(std::slice::from_ref(&committed_action)).expect("root");

        let result = driver
            .on_commit(NodeExecutionCommitContext {
                world_id: "w1".to_string(),
                node_id: "node-a".to_string(),
                height: 1,
                slot: 0,
                epoch: 0,
                node_block_hash: "node-h1".to_string(),
                action_root,
                committed_actions: vec![committed_action],
                committed_at_unix_ms: 1_000,
            })
            .expect("commit");

        assert_eq!(result.execution_height, 1);
        assert!(records_dir.join("00000000000000000001.json").exists());
        let _ = fs::remove_dir_all(dir);
    }
}
