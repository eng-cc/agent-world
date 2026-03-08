use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[cfg(test)]
use agent_world::consensus_action_payload::ConsensusActionPayloadEnvelope;
use agent_world::consensus_action_payload::{
    decode_consensus_action_payload, ConsensusActionPayloadBody,
};
use agent_world::runtime::{
    blake3_hex, BlobStore, LocalCasStore, ModuleRegistry, World as RuntimeWorld,
};
use agent_world::simulator::{
    Action as SimulatorAction, ActionSubmitter, WorldEventKind, WorldKernel,
};
use agent_world_node::{
    compute_consensus_action_root, NodeExecutionCommitContext, NodeExecutionCommitResult,
    NodeExecutionHook, NodeSnapshot,
};
use agent_world_wasm_abi::ModuleSandbox;
use agent_world_wasm_executor::{WasmExecutor, WasmExecutorConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionBridgeState {
    pub last_applied_committed_height: u64,
    pub last_execution_block_hash: Option<String>,
    pub last_execution_state_root: Option<String>,
    pub last_node_block_hash: Option<String>,
}

const EXECUTION_BRIDGE_RECORD_SCHEMA_V1: u32 = 1;
const EXECUTION_BRIDGE_RECORD_SCHEMA_V2: u32 = 2;
const EXECUTION_BRIDGE_DEFAULT_HOT_WINDOW_HEIGHTS: u64 = 32;
const EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS: u64 = 32;
const EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_KEEP_LATEST: usize = 4;

fn execution_bridge_record_schema_v1() -> u32 {
    EXECUTION_BRIDGE_RECORD_SCHEMA_V1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ExecutionBridgeRecordWire")]
pub(super) struct ExecutionBridgeRecord {
    pub schema_version: u32,
    pub world_id: String,
    pub height: u64,
    pub node_block_hash: Option<String>,
    pub execution_block_hash: String,
    pub execution_state_root: String,
    pub journal_len: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_state_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_log_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_effect_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulator_mirror: Option<ExecutionSimulatorMirrorRecord>,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ExecutionBridgeRecordWire {
    #[serde(default = "execution_bridge_record_schema_v1")]
    pub schema_version: u32,
    pub world_id: String,
    pub height: u64,
    #[serde(default)]
    pub node_block_hash: Option<String>,
    pub execution_block_hash: String,
    pub execution_state_root: String,
    pub journal_len: usize,
    #[serde(default)]
    pub latest_state_ref: Option<String>,
    #[serde(default)]
    pub snapshot_ref: Option<String>,
    #[serde(default)]
    pub journal_ref: Option<String>,
    #[serde(default)]
    pub commit_log_ref: Option<String>,
    #[serde(default)]
    pub checkpoint_ref: Option<String>,
    #[serde(default)]
    pub external_effect_ref: Option<String>,
    #[serde(default)]
    pub simulator_mirror: Option<ExecutionSimulatorMirrorRecord>,
    pub timestamp_ms: i64,
}

impl From<ExecutionBridgeRecordWire> for ExecutionBridgeRecord {
    fn from(record: ExecutionBridgeRecordWire) -> Self {
        let snapshot_ref = record.snapshot_ref;
        let latest_state_ref = record.latest_state_ref.or_else(|| snapshot_ref.clone());
        Self {
            schema_version: record.schema_version.max(EXECUTION_BRIDGE_RECORD_SCHEMA_V1),
            world_id: record.world_id,
            height: record.height,
            node_block_hash: record.node_block_hash,
            execution_block_hash: record.execution_block_hash,
            execution_state_root: record.execution_state_root,
            journal_len: record.journal_len,
            latest_state_ref,
            snapshot_ref,
            journal_ref: record.journal_ref,
            commit_log_ref: record.commit_log_ref,
            checkpoint_ref: record.checkpoint_ref,
            external_effect_ref: record.external_effect_ref,
            simulator_mirror: record.simulator_mirror,
            timestamp_ms: record.timestamp_ms,
        }
    }
}

impl ExecutionBridgeRecord {
    fn new_v2(
        world_id: String,
        height: u64,
        node_block_hash: Option<String>,
        execution_block_hash: String,
        execution_state_root: String,
        journal_len: usize,
        snapshot_ref: String,
        journal_ref: String,
        external_effect_ref: Option<String>,
        simulator_mirror: Option<ExecutionSimulatorMirrorRecord>,
        timestamp_ms: i64,
    ) -> Self {
        Self {
            schema_version: EXECUTION_BRIDGE_RECORD_SCHEMA_V2,
            world_id,
            height,
            node_block_hash,
            execution_block_hash,
            execution_state_root,
            journal_len,
            latest_state_ref: Some(snapshot_ref.clone()),
            snapshot_ref: Some(snapshot_ref),
            journal_ref: Some(journal_ref),
            commit_log_ref: None,
            checkpoint_ref: None,
            external_effect_ref,
            simulator_mirror,
            timestamp_ms,
        }
    }
}

const EXECUTION_CHECKPOINT_MANIFEST_SCHEMA_V1: u32 = 1;

fn execution_checkpoint_manifest_schema_v1() -> u32 {
    EXECUTION_CHECKPOINT_MANIFEST_SCHEMA_V1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionCheckpointManifest {
    #[serde(default = "execution_checkpoint_manifest_schema_v1")]
    pub schema_version: u32,
    pub checkpoint_id: String,
    pub world_id: String,
    pub height: u64,
    pub execution_block_hash: String,
    pub execution_state_root: String,
    pub latest_state_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pinned_refs: Vec<String>,
    pub manifest_hash: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ExecutionCheckpointLatestPointer {
    #[serde(default = "execution_checkpoint_manifest_schema_v1")]
    pub schema_version: u32,
    pub checkpoint_id: String,
    pub height: u64,
    pub manifest_hash: String,
    pub manifest_rel_path: String,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ExecutionCheckpointManifestHashPayload<'a> {
    schema_version: u32,
    checkpoint_id: &'a str,
    world_id: &'a str,
    height: u64,
    execution_block_hash: &'a str,
    execution_state_root: &'a str,
    latest_state_ref: &'a str,
    snapshot_ref: Option<&'a str>,
    journal_ref: Option<&'a str>,
    pinned_refs: &'a [String],
    created_at_ms: i64,
}

impl ExecutionCheckpointManifest {
    fn new(
        world_id: String,
        height: u64,
        execution_block_hash: String,
        execution_state_root: String,
        latest_state_ref: String,
        snapshot_ref: Option<String>,
        journal_ref: Option<String>,
        created_at_ms: i64,
    ) -> Result<Self, String> {
        let checkpoint_id = execution_checkpoint_id(height, execution_block_hash.as_str());
        let mut pinned_refs = vec![latest_state_ref.clone()];
        if let Some(snapshot_ref) = snapshot_ref.as_ref() {
            pinned_refs.push(snapshot_ref.clone());
        }
        if let Some(journal_ref) = journal_ref.as_ref() {
            pinned_refs.push(journal_ref.clone());
        }
        pinned_refs.sort();
        pinned_refs.dedup();

        let mut manifest = Self {
            schema_version: EXECUTION_CHECKPOINT_MANIFEST_SCHEMA_V1,
            checkpoint_id,
            world_id,
            height,
            execution_block_hash,
            execution_state_root,
            latest_state_ref,
            snapshot_ref,
            journal_ref,
            pinned_refs,
            manifest_hash: String::new(),
            created_at_ms,
        };
        manifest.manifest_hash = manifest.compute_manifest_hash()?;
        Ok(manifest)
    }

    fn compute_manifest_hash(&self) -> Result<String, String> {
        let payload = ExecutionCheckpointManifestHashPayload {
            schema_version: self.schema_version,
            checkpoint_id: self.checkpoint_id.as_str(),
            world_id: self.world_id.as_str(),
            height: self.height,
            execution_block_hash: self.execution_block_hash.as_str(),
            execution_state_root: self.execution_state_root.as_str(),
            latest_state_ref: self.latest_state_ref.as_str(),
            snapshot_ref: self.snapshot_ref.as_deref(),
            journal_ref: self.journal_ref.as_deref(),
            pinned_refs: self.pinned_refs.as_slice(),
            created_at_ms: self.created_at_ms,
        };
        Ok(blake3_hex(to_cbor(payload)?.as_slice()))
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version < EXECUTION_CHECKPOINT_MANIFEST_SCHEMA_V1 {
            return Err(format!(
                "execution checkpoint manifest {} has invalid schema_version={}",
                self.checkpoint_id, self.schema_version
            ));
        }
        if self.height == 0 {
            return Err(format!(
                "execution checkpoint manifest {} has invalid height=0",
                self.checkpoint_id
            ));
        }
        if self.latest_state_ref.is_empty() {
            return Err(format!(
                "execution checkpoint manifest {} missing latest_state_ref",
                self.checkpoint_id
            ));
        }
        let mut expected_pins = vec![self.latest_state_ref.clone()];
        if let Some(snapshot_ref) = self.snapshot_ref.as_ref() {
            expected_pins.push(snapshot_ref.clone());
        }
        if let Some(journal_ref) = self.journal_ref.as_ref() {
            expected_pins.push(journal_ref.clone());
        }
        expected_pins.sort();
        expected_pins.dedup();
        if expected_pins != self.pinned_refs {
            return Err(format!(
                "execution checkpoint manifest {} pin-set mismatch expected={:?} actual={:?}",
                self.checkpoint_id, expected_pins, self.pinned_refs
            ));
        }
        let expected_hash = self.compute_manifest_hash()?;
        if self.manifest_hash != expected_hash {
            return Err(format!(
                "execution checkpoint manifest {} hash mismatch expected={} actual={}",
                self.checkpoint_id, expected_hash, self.manifest_hash
            ));
        }
        Ok(())
    }
}

fn execution_checkpoint_id(height: u64, execution_block_hash: &str) -> String {
    let short_hash: String = execution_block_hash.chars().take(16).collect();
    format!("checkpoint-{:020}-{short_hash}", height)
}

fn execution_checkpoint_root_dir(execution_records_dir: &Path) -> std::path::PathBuf {
    execution_records_dir.join("checkpoints")
}

fn execution_checkpoint_manifest_path(
    execution_records_dir: &Path,
    height: u64,
) -> std::path::PathBuf {
    execution_checkpoint_root_dir(execution_records_dir)
        .join(format!("{:020}", height))
        .join("manifest.json")
}

fn execution_checkpoint_latest_path(execution_records_dir: &Path) -> std::path::PathBuf {
    execution_checkpoint_root_dir(execution_records_dir).join("latest.json")
}

fn execution_checkpoint_manifest_rel_path(height: u64) -> String {
    format!("{:020}/manifest.json", height)
}

fn list_execution_checkpoint_heights(execution_records_dir: &Path) -> Result<Vec<u64>, String> {
    let checkpoint_root = execution_checkpoint_root_dir(execution_records_dir);
    if !checkpoint_root.exists() {
        return Ok(Vec::new());
    }

    let mut heights = Vec::new();
    for entry in fs::read_dir(checkpoint_root.as_path()).map_err(|err| {
        format!(
            "read execution checkpoint root {} failed: {}",
            checkpoint_root.display(),
            err
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "read execution checkpoint dir entry under {} failed: {}",
                checkpoint_root.display(),
                err
            )
        })?;
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "read execution checkpoint dir entry type {} failed: {}",
                entry.path().display(),
                err
            )
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        let Ok(height) = name.parse::<u64>() else {
            continue;
        };
        if execution_checkpoint_manifest_path(execution_records_dir, height).exists() {
            heights.push(height);
        }
    }

    heights.sort_unstable();
    heights.dedup();
    Ok(heights)
}

fn persist_execution_checkpoint_manifest(
    execution_records_dir: &Path,
    manifest: &ExecutionCheckpointManifest,
) -> Result<(), String> {
    manifest.validate()?;
    let manifest_path = execution_checkpoint_manifest_path(execution_records_dir, manifest.height);
    let manifest_parent = manifest_path.parent().ok_or_else(|| {
        format!(
            "execution checkpoint manifest path {} missing parent",
            manifest_path.display()
        )
    })?;
    fs::create_dir_all(manifest_parent).map_err(|err| {
        format!(
            "create execution checkpoint dir {} failed: {}",
            manifest_parent.display(),
            err
        )
    })?;
    let manifest_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|err| format!("serialize execution checkpoint manifest failed: {}", err))?;
    super::write_bytes_atomic(manifest_path.as_path(), manifest_bytes.as_slice())?;

    let root_dir = execution_checkpoint_root_dir(execution_records_dir);
    fs::create_dir_all(root_dir.as_path()).map_err(|err| {
        format!(
            "create execution checkpoint root {} failed: {}",
            root_dir.display(),
            err
        )
    })?;
    let latest = ExecutionCheckpointLatestPointer {
        schema_version: EXECUTION_CHECKPOINT_MANIFEST_SCHEMA_V1,
        checkpoint_id: manifest.checkpoint_id.clone(),
        height: manifest.height,
        manifest_hash: manifest.manifest_hash.clone(),
        manifest_rel_path: execution_checkpoint_manifest_rel_path(manifest.height),
        updated_at_ms: manifest.created_at_ms,
    };
    let latest_bytes = serde_json::to_vec_pretty(&latest).map_err(|err| {
        format!(
            "serialize execution checkpoint latest pointer failed: {}",
            err
        )
    })?;
    let latest_path = execution_checkpoint_latest_path(execution_records_dir);
    super::write_bytes_atomic(latest_path.as_path(), latest_bytes.as_slice())
}

fn load_execution_checkpoint_manifest(path: &Path) -> Result<ExecutionCheckpointManifest, String> {
    let bytes = fs::read(path).map_err(|err| {
        format!(
            "read execution checkpoint manifest {} failed: {}",
            path.display(),
            err
        )
    })?;
    let manifest = serde_json::from_slice::<ExecutionCheckpointManifest>(bytes.as_slice())
        .map_err(|err| {
            format!(
                "parse execution checkpoint manifest {} failed: {}",
                path.display(),
                err
            )
        })?;
    manifest.validate()?;
    Ok(manifest)
}

fn load_latest_execution_checkpoint_manifest(
    execution_records_dir: &Path,
) -> Result<Option<ExecutionCheckpointManifest>, String> {
    let latest_path = execution_checkpoint_latest_path(execution_records_dir);
    if !latest_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(latest_path.as_path()).map_err(|err| {
        format!(
            "read execution checkpoint latest pointer {} failed: {}",
            latest_path.display(),
            err
        )
    })?;
    let latest = serde_json::from_slice::<ExecutionCheckpointLatestPointer>(bytes.as_slice())
        .map_err(|err| {
            format!(
                "parse execution checkpoint latest pointer {} failed: {}",
                latest_path.display(),
                err
            )
        })?;
    let manifest_path =
        execution_checkpoint_root_dir(execution_records_dir).join(latest.manifest_rel_path);
    let manifest = load_execution_checkpoint_manifest(manifest_path.as_path())?;
    if manifest.height != latest.height {
        return Err(format!(
            "execution checkpoint latest pointer height mismatch expected={} actual={}",
            latest.height, manifest.height
        ));
    }
    if manifest.manifest_hash != latest.manifest_hash {
        return Err(format!(
            "execution checkpoint latest pointer hash mismatch expected={} actual={}",
            latest.manifest_hash, manifest.manifest_hash
        ));
    }
    if manifest.checkpoint_id != latest.checkpoint_id {
        return Err(format!(
            "execution checkpoint latest pointer id mismatch expected={} actual={}",
            latest.checkpoint_id, manifest.checkpoint_id
        ));
    }
    Ok(Some(manifest))
}

fn execution_bridge_record_path(execution_records_dir: &Path, height: u64) -> std::path::PathBuf {
    execution_records_dir.join(format!("{:020}.json", height))
}

fn list_execution_bridge_record_heights(execution_records_dir: &Path) -> Result<Vec<u64>, String> {
    if !execution_records_dir.exists() {
        return Ok(Vec::new());
    }

    let mut heights = Vec::new();
    for entry in fs::read_dir(execution_records_dir).map_err(|err| {
        format!(
            "read execution records dir {} failed: {}",
            execution_records_dir.display(),
            err
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "read execution record dir entry under {} failed: {}",
                execution_records_dir.display(),
                err
            )
        })?;
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "read execution record dir entry type {} failed: {}",
                entry.path().display(),
                err
            )
        })?;
        if !file_type.is_file() {
            continue;
        }
        let Some(file_name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if file_name == "latest.json" || !file_name.ends_with(".json") {
            continue;
        }
        let Some(stem) = file_name.strip_suffix(".json") else {
            continue;
        };
        let Ok(height) = stem.parse::<u64>() else {
            continue;
        };
        heights.push(height);
    }

    heights.sort_unstable();
    heights.dedup();
    Ok(heights)
}

fn maybe_insert_pin_ref(pinned_refs: &mut BTreeSet<String>, content_ref: Option<&str>) {
    if let Some(content_ref) = content_ref.filter(|content_ref| !content_ref.is_empty()) {
        pinned_refs.insert(content_ref.to_string());
    }
}

fn collect_execution_bridge_record_retained_refs(
    record: &ExecutionBridgeRecord,
    retain_latest_head: bool,
    retain_hot_window: bool,
    pinned_refs: &mut BTreeSet<String>,
) {
    maybe_insert_pin_ref(pinned_refs, record.commit_log_ref.as_deref());
    maybe_insert_pin_ref(pinned_refs, record.external_effect_ref.as_deref());

    if retain_latest_head {
        maybe_insert_pin_ref(pinned_refs, record.latest_state_ref.as_deref());
    }
    if retain_latest_head || retain_hot_window {
        maybe_insert_pin_ref(pinned_refs, record.snapshot_ref.as_deref());
        maybe_insert_pin_ref(pinned_refs, record.journal_ref.as_deref());
        if let Some(simulator_mirror) = record.simulator_mirror.as_ref() {
            pinned_refs.insert(simulator_mirror.snapshot_ref.clone());
            pinned_refs.insert(simulator_mirror.journal_ref.clone());
        }
    }
}

fn collect_execution_checkpoint_retained_refs(
    execution_records_dir: &Path,
    pinned_refs: &mut BTreeSet<String>,
) -> Result<(), String> {
    let checkpoint_root = execution_checkpoint_root_dir(execution_records_dir);
    if !checkpoint_root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(checkpoint_root.as_path()).map_err(|err| {
        format!(
            "read execution checkpoint root {} failed: {}",
            checkpoint_root.display(),
            err
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "read execution checkpoint dir entry under {} failed: {}",
                checkpoint_root.display(),
                err
            )
        })?;
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "read execution checkpoint dir entry type {} failed: {}",
                entry.path().display(),
                err
            )
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = load_execution_checkpoint_manifest(manifest_path.as_path())?;
        pinned_refs.extend(manifest.pinned_refs);
    }

    Ok(())
}

fn build_execution_bridge_pin_set(
    execution_records_dir: &Path,
    hot_window_heights: u64,
) -> Result<ExecutionBridgePinSet, String> {
    let record_heights = list_execution_bridge_record_heights(execution_records_dir)?;
    let checkpoint_heights = list_execution_checkpoint_heights(execution_records_dir)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let Some(latest_height) = record_heights.last().copied() else {
        let mut pin_set = ExecutionBridgePinSet {
            checkpoint_heights,
            ..ExecutionBridgePinSet::default()
        };
        collect_execution_checkpoint_retained_refs(
            execution_records_dir,
            &mut pin_set.pinned_refs,
        )?;
        return Ok(pin_set);
    };

    let retained_hot_window = hot_window_heights.max(1);
    let hot_window_start_height =
        latest_height.saturating_sub(retained_hot_window.saturating_sub(1));
    let mut pin_set = ExecutionBridgePinSet {
        latest_height: Some(latest_height),
        hot_window_start_height: Some(hot_window_start_height),
        checkpoint_heights,
        pinned_refs: BTreeSet::new(),
    };

    for height in record_heights {
        let record = load_execution_bridge_record(
            execution_bridge_record_path(execution_records_dir, height).as_path(),
        )?;
        collect_execution_bridge_record_retained_refs(
            &record,
            record.height == latest_height,
            record.height >= hot_window_start_height,
            &mut pin_set.pinned_refs,
        );
    }
    collect_execution_checkpoint_retained_refs(execution_records_dir, &mut pin_set.pinned_refs)?;

    Ok(pin_set)
}

fn sync_execution_bridge_pin_set(
    execution_records_dir: &Path,
    execution_store: &LocalCasStore,
    hot_window_heights: u64,
) -> Result<ExecutionBridgePinSet, String> {
    let pin_set = build_execution_bridge_pin_set(execution_records_dir, hot_window_heights)?;
    let current_pins = execution_store
        .list_pins()
        .map_err(|err| format!("list execution store pins failed: {:?}", err))?
        .into_iter()
        .collect::<BTreeSet<_>>();

    for stale_ref in current_pins.difference(&pin_set.pinned_refs) {
        execution_store
            .unpin(stale_ref.as_str())
            .map_err(|err| format!("unpin execution store ref {} failed: {:?}", stale_ref, err))?;
    }
    for pinned_ref in pin_set.pinned_refs.difference(&current_pins) {
        execution_store
            .pin(pinned_ref.as_str())
            .map_err(|err| format!("pin execution store ref {} failed: {:?}", pinned_ref, err))?;
    }

    Ok(pin_set)
}

fn list_execution_bridge_legacy_heights(execution_records_dir: &Path) -> Result<Vec<u64>, String> {
    let mut legacy_heights = Vec::new();
    for height in list_execution_bridge_record_heights(execution_records_dir)? {
        let record = load_execution_bridge_record(
            execution_bridge_record_path(execution_records_dir, height).as_path(),
        )?;
        if record.schema_version < EXECUTION_BRIDGE_RECORD_SCHEMA_V2 {
            legacy_heights.push(height);
        }
    }
    Ok(legacy_heights)
}

fn update_execution_bridge_record_checkpoint_ref(
    execution_records_dir: &Path,
    height: u64,
    checkpoint_ref: Option<String>,
) -> Result<(), String> {
    let path = execution_bridge_record_path(execution_records_dir, height);
    if !path.exists() {
        return Ok(());
    }
    let mut record = load_execution_bridge_record(path.as_path())?;
    record.checkpoint_ref = checkpoint_ref;
    persist_execution_bridge_record_only(execution_records_dir, &record)?;
    Ok(())
}

fn prune_execution_checkpoint_manifests(
    execution_records_dir: &Path,
    checkpoint_keep_latest: usize,
) -> Result<Vec<u64>, String> {
    let checkpoint_heights = list_execution_checkpoint_heights(execution_records_dir)?;
    let retained_count = checkpoint_keep_latest.max(1);
    if checkpoint_heights.len() <= retained_count {
        return Ok(Vec::new());
    }

    let prune_heights = checkpoint_heights[..checkpoint_heights.len() - retained_count].to_vec();
    for height in &prune_heights {
        let manifest_path = execution_checkpoint_manifest_path(execution_records_dir, *height);
        let manifest_dir = manifest_path.parent().ok_or_else(|| {
            format!(
                "execution checkpoint manifest path {} missing parent",
                manifest_path.display()
            )
        })?;
        if manifest_dir.exists() {
            fs::remove_dir_all(manifest_dir).map_err(|err| {
                format!(
                    "remove execution checkpoint dir {} failed: {}",
                    manifest_dir.display(),
                    err
                )
            })?;
        }
        update_execution_bridge_record_checkpoint_ref(execution_records_dir, *height, None)?;
    }

    Ok(prune_heights)
}

fn maybe_persist_execution_checkpoint_for_record(
    execution_records_dir: &Path,
    record: &ExecutionBridgeRecord,
    checkpoint_interval_heights: u64,
    checkpoint_keep_latest: usize,
) -> Result<Option<String>, String> {
    if checkpoint_interval_heights == 0
        || record.height == 0
        || record.height % checkpoint_interval_heights != 0
    {
        return Ok(None);
    }

    let latest_state_ref = record.latest_state_ref.clone().ok_or_else(|| {
        format!(
            "execution checkpoint height {} missing latest_state_ref",
            record.height
        )
    })?;
    let manifest = ExecutionCheckpointManifest::new(
        record.world_id.clone(),
        record.height,
        record.execution_block_hash.clone(),
        record.execution_state_root.clone(),
        latest_state_ref,
        record.snapshot_ref.clone(),
        record.journal_ref.clone(),
        record.timestamp_ms,
    )?;
    persist_execution_checkpoint_manifest(execution_records_dir, &manifest)?;
    let checkpoint_ref = execution_checkpoint_manifest_rel_path(record.height);
    prune_execution_checkpoint_manifests(execution_records_dir, checkpoint_keep_latest)?;
    Ok(Some(checkpoint_ref))
}

fn compact_execution_bridge_records(
    execution_records_dir: &Path,
    pin_set: &ExecutionBridgePinSet,
) -> Result<usize, String> {
    let record_heights = list_execution_bridge_record_heights(execution_records_dir)?;
    let latest_height = pin_set.latest_height;
    let hot_window_start_height = pin_set.hot_window_start_height.unwrap_or(u64::MAX);
    let mut rewritten_records = 0_usize;

    for height in record_heights {
        let path = execution_bridge_record_path(execution_records_dir, height);
        let mut record = load_execution_bridge_record(path.as_path())?;
        let original_record = record.clone();
        let retain_latest_head = latest_height == Some(height);
        let retain_hot_window = height >= hot_window_start_height;
        let retain_checkpoint = pin_set.checkpoint_heights.contains(&height);

        if !retain_latest_head && !retain_hot_window {
            record.latest_state_ref = None;
            record.snapshot_ref = None;
            record.journal_ref = None;
            record.simulator_mirror = None;
            if !retain_checkpoint {
                record.checkpoint_ref = None;
            }
        }

        if record != original_record {
            if retain_latest_head {
                persist_execution_bridge_record(execution_records_dir, &record)?;
            } else {
                persist_execution_bridge_record_only(execution_records_dir, &record)?;
            }
            rewritten_records = rewritten_records.saturating_add(1);
        }
    }

    Ok(rewritten_records)
}

fn run_execution_bridge_retention_maintenance(
    execution_records_dir: &Path,
    execution_store: &LocalCasStore,
    hot_window_heights: u64,
) -> Result<u64, String> {
    let legacy_heights = list_execution_bridge_legacy_heights(execution_records_dir)?;
    if !legacy_heights.is_empty() {
        sync_execution_bridge_pin_set(execution_records_dir, execution_store, hot_window_heights)?;
        return Ok(0);
    }

    let pin_set = build_execution_bridge_pin_set(execution_records_dir, hot_window_heights)?;
    compact_execution_bridge_records(execution_records_dir, &pin_set)?;
    sync_execution_bridge_pin_set(execution_records_dir, execution_store, hot_window_heights)?;
    execution_store
        .prune_orphan_blobs()
        .map_err(|err| format!("prune execution store orphan blobs failed: {:?}", err))
}

const EXECUTION_EXTERNAL_EFFECT_SCHEMA_V1: u32 = 1;
const EXECUTION_EXTERNAL_EFFECT_CONTRACT_CLOSED_WORLD_V1: &str = "closed_world_v1";

fn execution_external_effect_schema_v1() -> u32 {
    EXECUTION_EXTERNAL_EFFECT_SCHEMA_V1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionExternalEffectMaterialization {
    #[serde(default = "execution_external_effect_schema_v1")]
    pub schema_version: u32,
    pub contract: String,
    pub world_id: String,
    pub node_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub node_block_hash: String,
    pub action_root: String,
    pub committed_at_unix_ms: i64,
    pub pre_step_execution_state_root: String,
    pub world_manifest_hash: String,
    pub active_modules_hash: String,
    pub committed_actions_hash: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_modules: Vec<ExecutionModuleResolutionAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub committed_actions: Vec<ExecutionCommittedActionAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionModuleResolutionAnchor {
    pub instance_id: String,
    pub module_id: String,
    pub module_version: String,
    pub wasm_hash: String,
    pub install_target: agent_world::simulator::ModuleInstallTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionCommittedActionAnchor {
    pub action_id: u64,
    pub submitter_player_id: String,
    pub payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecutionReplayRecordInput {
    pub record: ExecutionBridgeRecord,
    pub external_effect: Option<ExecutionExternalEffectMaterialization>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecutionReplayPlan {
    pub target_height: u64,
    pub start_height: u64,
    pub checkpoint: Option<ExecutionCheckpointManifest>,
    pub records: Vec<ExecutionReplayRecordInput>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExecutionBridgePinSet {
    latest_height: Option<u64>,
    hot_window_start_height: Option<u64>,
    checkpoint_heights: BTreeSet<u64>,
    pinned_refs: BTreeSet<String>,
}

impl ExecutionExternalEffectMaterialization {
    fn validate(&self) -> Result<(), String> {
        if self.schema_version < EXECUTION_EXTERNAL_EFFECT_SCHEMA_V1 {
            return Err(format!(
                "execution external effect has invalid schema_version={} at height={}",
                self.schema_version, self.height
            ));
        }
        if self.contract != EXECUTION_EXTERNAL_EFFECT_CONTRACT_CLOSED_WORLD_V1 {
            return Err(format!(
                "execution external effect has unsupported contract={} at height={}",
                self.contract, self.height
            ));
        }
        if self.world_id.trim().is_empty()
            || self.node_id.trim().is_empty()
            || self.node_block_hash.trim().is_empty()
            || self.action_root.trim().is_empty()
            || self.pre_step_execution_state_root.trim().is_empty()
            || self.world_manifest_hash.trim().is_empty()
        {
            return Err(format!(
                "execution external effect missing required fields at height={}",
                self.height
            ));
        }
        if !self.unresolved_inputs.is_empty() {
            return Err(format!(
                "execution external effect unresolved inputs at height={} inputs={:?}",
                self.height, self.unresolved_inputs
            ));
        }
        let expected_active_hash = execution_module_anchor_hash(self.active_modules.as_slice())?;
        if self.active_modules_hash != expected_active_hash {
            return Err(format!(
                "execution external effect active_modules_hash mismatch expected={} actual={} height={}",
                expected_active_hash, self.active_modules_hash, self.height
            ));
        }
        let expected_actions_hash =
            execution_committed_actions_hash(self.committed_actions.as_slice())?;
        if self.committed_actions_hash != expected_actions_hash {
            return Err(format!(
                "execution external effect committed_actions_hash mismatch expected={} actual={} height={}",
                expected_actions_hash, self.committed_actions_hash, self.height
            ));
        }
        Ok(())
    }
}

fn execution_module_anchor_hash(
    anchors: &[ExecutionModuleResolutionAnchor],
) -> Result<String, String> {
    Ok(blake3_hex(to_cbor(anchors)?.as_slice()))
}

fn execution_committed_actions_hash(
    actions: &[ExecutionCommittedActionAnchor],
) -> Result<String, String> {
    Ok(blake3_hex(to_cbor(actions)?.as_slice()))
}

fn collect_execution_module_resolution_anchors(
    execution_world: &RuntimeWorld,
) -> Result<Vec<ExecutionModuleResolutionAnchor>, String> {
    let module_registry = execution_world.module_registry();
    let state = execution_world.state();
    let mut anchors = Vec::new();
    let mut module_ids_with_instances = BTreeSet::new();
    for instance in state.module_instances.values() {
        module_ids_with_instances.insert(instance.module_id.clone());
        let key = ModuleRegistry::record_key(
            instance.module_id.as_str(),
            instance.module_version.as_str(),
        );
        let record = module_registry.records.get(&key).ok_or_else(|| {
            format!(
                "execution external effect missing module record {} for instance {}",
                key, instance.instance_id
            )
        })?;
        anchors.push(ExecutionModuleResolutionAnchor {
            instance_id: instance.instance_id.clone(),
            module_id: instance.module_id.clone(),
            module_version: record.manifest.version.clone(),
            wasm_hash: record.manifest.wasm_hash.clone(),
            install_target: instance.install_target.clone(),
        });
    }

    let mut legacy_module_ids: Vec<String> = module_registry.active.keys().cloned().collect();
    legacy_module_ids.sort();
    for module_id in legacy_module_ids {
        if module_ids_with_instances.contains(&module_id) {
            continue;
        }
        let version = module_registry.active.get(&module_id).ok_or_else(|| {
            format!(
                "execution external effect missing active module version for {}",
                module_id
            )
        })?;
        let key = ModuleRegistry::record_key(module_id.as_str(), version.as_str());
        let record = module_registry
            .records
            .get(&key)
            .ok_or_else(|| format!("execution external effect missing module record {}", key))?;
        anchors.push(ExecutionModuleResolutionAnchor {
            instance_id: module_id.clone(),
            module_id: module_id.clone(),
            module_version: version.clone(),
            wasm_hash: record.manifest.wasm_hash.clone(),
            install_target: state
                .installed_module_targets
                .get(&module_id)
                .cloned()
                .unwrap_or_default(),
        });
    }

    anchors.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    Ok(anchors)
}

fn collect_execution_committed_action_anchors(
    context: &NodeExecutionCommitContext,
) -> Vec<ExecutionCommittedActionAnchor> {
    let mut anchors: Vec<_> = context
        .committed_actions
        .iter()
        .map(|action| ExecutionCommittedActionAnchor {
            action_id: action.action_id,
            submitter_player_id: action.submitter_player_id.clone(),
            payload_hash: action.payload_hash.clone(),
        })
        .collect();
    anchors.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    anchors
}

fn build_execution_external_effect_materialization(
    execution_world: &RuntimeWorld,
    context: &NodeExecutionCommitContext,
) -> Result<ExecutionExternalEffectMaterialization, String> {
    let pre_step_snapshot = execution_world.snapshot();
    let pre_step_execution_state_root = blake3_hex(to_cbor(pre_step_snapshot)?.as_slice());
    let world_manifest_hash = execution_world
        .current_manifest_hash()
        .map_err(|err| format!("execution external effect manifest hash failed: {:?}", err))?;
    let active_modules = collect_execution_module_resolution_anchors(execution_world)?;
    let committed_actions = collect_execution_committed_action_anchors(context);
    let materialization = ExecutionExternalEffectMaterialization {
        schema_version: EXECUTION_EXTERNAL_EFFECT_SCHEMA_V1,
        contract: EXECUTION_EXTERNAL_EFFECT_CONTRACT_CLOSED_WORLD_V1.to_string(),
        world_id: context.world_id.clone(),
        node_id: context.node_id.clone(),
        height: context.height,
        slot: context.slot,
        epoch: context.epoch,
        node_block_hash: context.node_block_hash.clone(),
        action_root: context.action_root.clone(),
        committed_at_unix_ms: context.committed_at_unix_ms,
        pre_step_execution_state_root,
        world_manifest_hash,
        active_modules_hash: execution_module_anchor_hash(active_modules.as_slice())?,
        committed_actions_hash: execution_committed_actions_hash(committed_actions.as_slice())?,
        active_modules,
        committed_actions,
        unresolved_inputs: Vec::new(),
    };
    materialization.validate()?;
    Ok(materialization)
}

fn persist_execution_external_effect_materialization(
    execution_store: &LocalCasStore,
    materialization: &ExecutionExternalEffectMaterialization,
) -> Result<String, String> {
    materialization.validate()?;
    let bytes = to_cbor(materialization)?;
    execution_store
        .put_bytes(bytes.as_slice())
        .map_err(|err| format!("execution external effect CAS put failed: {:?}", err))
}

fn load_execution_external_effect_materialization(
    execution_store: &LocalCasStore,
    external_effect_ref: &str,
) -> Result<ExecutionExternalEffectMaterialization, String> {
    let bytes = execution_store.get(external_effect_ref).map_err(|err| {
        format!(
            "execution external effect CAS get failed ref={} err={:?}",
            external_effect_ref, err
        )
    })?;
    let materialization =
        serde_cbor::from_slice::<ExecutionExternalEffectMaterialization>(bytes.as_slice())
            .map_err(|err| {
                format!(
                    "parse execution external effect failed ref={} err={}",
                    external_effect_ref, err
                )
            })?;
    materialization.validate()?;
    Ok(materialization)
}

fn load_execution_replay_record_input(
    execution_store: &LocalCasStore,
    record: ExecutionBridgeRecord,
) -> Result<ExecutionReplayRecordInput, String> {
    let external_effect = match record.external_effect_ref.as_deref() {
        Some(external_effect_ref) => {
            let external_effect = load_execution_external_effect_materialization(
                execution_store,
                external_effect_ref,
            )?;
            if external_effect.world_id != record.world_id {
                return Err(format!(
                    "execution replay input world_id mismatch height={} expected={} actual={}",
                    record.height, record.world_id, external_effect.world_id
                ));
            }
            if external_effect.height != record.height {
                return Err(format!(
                    "execution replay input height mismatch expected={} actual={}",
                    record.height, external_effect.height
                ));
            }
            if let Some(node_block_hash) = record.node_block_hash.as_deref() {
                if external_effect.node_block_hash != node_block_hash {
                    return Err(format!(
                        "execution replay input node_block_hash mismatch height={} expected={} actual={}",
                        record.height, node_block_hash, external_effect.node_block_hash
                    ));
                }
            }
            Some(external_effect)
        }
        None => None,
    };
    Ok(ExecutionReplayRecordInput {
        record,
        external_effect,
    })
}

fn load_execution_bridge_record(path: &Path) -> Result<ExecutionBridgeRecord, String> {
    let bytes = fs::read(path).map_err(|err| {
        format!(
            "read execution bridge record {} failed: {}",
            path.display(),
            err
        )
    })?;
    serde_json::from_slice::<ExecutionBridgeRecord>(bytes.as_slice()).map_err(|err| {
        format!(
            "parse execution bridge record {} failed: {}",
            path.display(),
            err
        )
    })
}

fn find_nearest_execution_checkpoint_manifest(
    execution_records_dir: &Path,
    target_height: u64,
) -> Result<Option<ExecutionCheckpointManifest>, String> {
    if target_height == 0 {
        return Ok(None);
    }
    if let Some(latest) = load_latest_execution_checkpoint_manifest(execution_records_dir)? {
        if latest.height <= target_height {
            return Ok(Some(latest));
        }
    }
    let checkpoint_root = execution_checkpoint_root_dir(execution_records_dir);
    if !checkpoint_root.exists() {
        return Ok(None);
    }
    let mut best_height = None;
    for entry in fs::read_dir(checkpoint_root.as_path()).map_err(|err| {
        format!(
            "read execution checkpoint root {} failed: {}",
            checkpoint_root.display(),
            err
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "read execution checkpoint dir entry under {} failed: {}",
                checkpoint_root.display(),
                err
            )
        })?;
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "read execution checkpoint dir entry type {} failed: {}",
                entry.path().display(),
                err
            )
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        let Ok(height) = name.parse::<u64>() else {
            continue;
        };
        if height <= target_height && best_height.map(|best| height > best).unwrap_or(true) {
            best_height = Some(height);
        }
    }
    let Some(best_height) = best_height else {
        return Ok(None);
    };
    load_execution_checkpoint_manifest(
        execution_checkpoint_manifest_path(execution_records_dir, best_height).as_path(),
    )
    .map(Some)
}

fn build_execution_replay_plan(
    execution_records_dir: &Path,
    execution_store: &LocalCasStore,
    target_height: u64,
) -> Result<ExecutionReplayPlan, String> {
    if target_height == 0 {
        return Ok(ExecutionReplayPlan {
            target_height,
            start_height: 0,
            checkpoint: None,
            records: Vec::new(),
        });
    }
    let checkpoint =
        find_nearest_execution_checkpoint_manifest(execution_records_dir, target_height)?;
    let start_height = checkpoint
        .as_ref()
        .map(|manifest| manifest.height.saturating_add(1))
        .unwrap_or(1);
    let mut records = Vec::new();
    if start_height <= target_height {
        for height in start_height..=target_height {
            let path = execution_records_dir.join(format!("{:020}.json", height));
            if !path.exists() {
                return Err(format!(
                    "execution replay plan missing commit record for height {} at {}",
                    height,
                    path.display()
                ));
            }
            let record = load_execution_bridge_record(path.as_path())?;
            if record.height != height {
                return Err(format!(
                    "execution replay plan height mismatch expected={} actual={} path={}",
                    height,
                    record.height,
                    path.display()
                ));
            }
            records.push(load_execution_replay_record_input(execution_store, record)?);
        }
    }
    Ok(ExecutionReplayPlan {
        target_height,
        start_height,
        checkpoint,
        records,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ExecutionSimulatorMirrorRecord {
    pub action_count: usize,
    pub rejected_action_count: usize,
    pub journal_len: usize,
    pub snapshot_ref: String,
    pub journal_ref: String,
    pub state_root: String,
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
    simulator_world_dir: std::path::PathBuf,
    execution_store: LocalCasStore,
    state: ExecutionBridgeState,
    execution_world: RuntimeWorld,
    simulator_mirror: WorldKernel,
    execution_sandbox: Box<dyn ModuleSandbox + Send>,
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
        let execution_sandbox: Box<dyn ModuleSandbox + Send> =
            Box::new(WasmExecutor::new(WasmExecutorConfig::default()));
        let mut driver = Self::new_with_sandbox(
            state_path,
            world_dir,
            records_dir,
            storage_root,
            state,
            execution_world,
            execution_sandbox,
        );
        driver.simulator_mirror =
            load_simulator_execution_world(driver.simulator_world_dir.as_path())?;
        Ok(driver)
    }

    fn new_with_sandbox(
        state_path: std::path::PathBuf,
        world_dir: std::path::PathBuf,
        records_dir: std::path::PathBuf,
        storage_root: std::path::PathBuf,
        state: ExecutionBridgeState,
        execution_world: RuntimeWorld,
        execution_sandbox: Box<dyn ModuleSandbox + Send>,
    ) -> Self {
        let simulator_world_dir = simulator_world_dir_from_execution_world_dir(world_dir.as_path());
        Self {
            state_path,
            world_dir,
            records_dir,
            simulator_world_dir,
            execution_store: LocalCasStore::new(storage_root),
            state,
            execution_world,
            simulator_mirror: WorldKernel::new(),
            execution_sandbox,
        }
    }

    fn apply_simulator_actions(
        &mut self,
        height: u64,
        simulator_actions: &[(SimulatorAction, ActionSubmitter)],
    ) -> Result<Option<ExecutionSimulatorMirrorRecord>, String> {
        if simulator_actions.is_empty() {
            return Ok(None);
        }

        let mut rejected_action_count = 0_usize;
        for (action, submitter) in simulator_actions {
            match submitter {
                ActionSubmitter::System => {
                    self.simulator_mirror
                        .submit_action_from_system(action.clone());
                }
                ActionSubmitter::Agent { agent_id } => {
                    self.simulator_mirror
                        .submit_action_from_agent(agent_id.clone(), action.clone());
                }
                ActionSubmitter::Player { player_id } => {
                    self.simulator_mirror
                        .submit_action_from_player(player_id.clone(), action.clone());
                }
            }

            let event = self.simulator_mirror.step().ok_or_else(|| {
                format!(
                    "execution driver simulator mirror step produced no event at height={height}"
                )
            })?;
            if matches!(event.kind, WorldEventKind::ActionRejected { .. }) {
                rejected_action_count = rejected_action_count.saturating_add(1);
            }
        }

        let snapshot_value = self.simulator_mirror.snapshot();
        let journal_value = self.simulator_mirror.journal_snapshot();
        let snapshot_bytes = to_cbor(snapshot_value)?;
        let journal_bytes = to_cbor(journal_value)?;

        let snapshot_ref = self
            .execution_store
            .put_bytes(snapshot_bytes.as_slice())
            .map_err(|err| {
                format!(
                    "execution driver simulator CAS snapshot put failed: {:?}",
                    err
                )
            })?;
        let journal_ref = self
            .execution_store
            .put_bytes(journal_bytes.as_slice())
            .map_err(|err| {
                format!(
                    "execution driver simulator CAS journal put failed: {:?}",
                    err
                )
            })?;
        let state_root = blake3_hex(snapshot_bytes.as_slice());
        persist_simulator_execution_world(
            self.simulator_world_dir.as_path(),
            &self.simulator_mirror,
        )?;

        Ok(Some(ExecutionSimulatorMirrorRecord {
            action_count: simulator_actions.len(),
            rejected_action_count,
            journal_len: self.simulator_mirror.journal().len(),
            snapshot_ref,
            journal_ref,
            state_root,
        }))
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
        let next_expected_height = self.state.last_applied_committed_height.saturating_add(1);
        if context.height != next_expected_height {
            eprintln!(
                "execution driver detected non-contiguous committed heights: last_applied={} incoming={} (continuing with gap)",
                self.state.last_applied_committed_height, context.height
            );
            self.state.last_applied_committed_height = context.height.saturating_sub(1);
            self.state.last_node_block_hash = None;
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

        let external_effect =
            build_execution_external_effect_materialization(&self.execution_world, &context)?;
        let external_effect_ref = persist_execution_external_effect_materialization(
            &self.execution_store,
            &external_effect,
        )?;

        let mut decoded_runtime_actions = Vec::with_capacity(context.committed_actions.len());
        let mut decoded_simulator_actions = Vec::with_capacity(context.committed_actions.len());
        for action in &context.committed_actions {
            match decode_consensus_action_payload(action.payload_cbor.as_slice()) {
                Ok(ConsensusActionPayloadBody::RuntimeAction { action: decoded }) => {
                    decoded_runtime_actions.push(decoded);
                }
                Ok(ConsensusActionPayloadBody::SimulatorAction { action, submitter }) => {
                    decoded_simulator_actions.push((action, submitter));
                }
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

        for action in decoded_runtime_actions {
            self.execution_world.submit_action(action);
        }
        self.execution_world
            .step_with_modules(&mut *self.execution_sandbox)
            .map_err(|err| {
                format!(
                    "execution driver world.step failed at height {}: {:?}",
                    context.height, err
                )
            })?;
        let simulator_mirror =
            self.apply_simulator_actions(context.height, decoded_simulator_actions.as_slice())?;

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

        let mut record = ExecutionBridgeRecord::new_v2(
            context.world_id.clone(),
            context.height,
            node_block_hash.clone(),
            execution_block_hash.clone(),
            execution_state_root.clone(),
            self.execution_world.journal().len(),
            snapshot_ref,
            journal_ref,
            Some(external_effect_ref),
            simulator_mirror,
            context.committed_at_unix_ms,
        );
        record.checkpoint_ref = maybe_persist_execution_checkpoint_for_record(
            self.records_dir.as_path(),
            &record,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_KEEP_LATEST,
        )?;
        persist_execution_bridge_record(self.records_dir.as_path(), &record)?;

        self.state.last_applied_committed_height = context.height;
        self.state.last_execution_block_hash = Some(execution_block_hash);
        self.state.last_execution_state_root = Some(execution_state_root);
        self.state.last_node_block_hash = node_block_hash;

        persist_execution_bridge_state(self.state_path.as_path(), &self.state)?;
        persist_execution_world(self.world_dir.as_path(), &self.execution_world)?;
        if let Err(err) = run_execution_bridge_retention_maintenance(
            self.records_dir.as_path(),
            &self.execution_store,
            EXECUTION_BRIDGE_DEFAULT_HOT_WINDOW_HEIGHTS,
        ) {
            eprintln!(
                "execution driver retention pin-set sync failed at height {}: {}",
                context.height, err
            );
        }

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

fn simulator_world_dir_from_execution_world_dir(world_dir: &Path) -> std::path::PathBuf {
    match world_dir.file_name().and_then(|name| name.to_str()) {
        Some(name) if !name.is_empty() => {
            world_dir.with_file_name(format!("{name}-simulator-mirror"))
        }
        _ => world_dir.join("simulator-mirror"),
    }
}

fn load_simulator_execution_world(world_dir: &Path) -> Result<WorldKernel, String> {
    let snapshot_path = world_dir.join("snapshot.json");
    let journal_path = world_dir.join("journal.json");
    if !snapshot_path.exists() || !journal_path.exists() {
        return Ok(WorldKernel::new());
    }
    WorldKernel::load_from_dir(world_dir).map_err(|err| {
        format!(
            "load simulator execution mirror from {} failed: {:?}",
            world_dir.display(),
            err
        )
    })
}

fn persist_simulator_execution_world(
    world_dir: &Path,
    simulator_world: &WorldKernel,
) -> Result<(), String> {
    simulator_world.save_to_dir(world_dir).map_err(|err| {
        format!(
            "save simulator execution mirror to {} failed: {:?}",
            world_dir.display(),
            err
        )
    })
}

pub(super) fn bridge_committed_heights(
    snapshot: &NodeSnapshot,
    observed_at_unix_ms: i64,
    execution_world: &mut RuntimeWorld,
    execution_sandbox: &mut dyn ModuleSandbox,
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
        execution_world
            .step_with_modules(execution_sandbox)
            .map_err(|err| {
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

        let mut record = ExecutionBridgeRecord::new_v2(
            snapshot.world_id.clone(),
            height,
            node_block_hash.clone(),
            execution_block_hash.clone(),
            execution_state_root.clone(),
            execution_world.journal().len(),
            snapshot_ref,
            journal_ref,
            None,
            None,
            observed_at_unix_ms,
        );
        record.checkpoint_ref = maybe_persist_execution_checkpoint_for_record(
            execution_records_dir,
            &record,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_KEEP_LATEST,
        )?;
        persist_execution_bridge_record(execution_records_dir, &record)?;

        state.last_applied_committed_height = height;
        state.last_execution_block_hash = Some(execution_block_hash);
        state.last_execution_state_root = Some(execution_state_root);
        state.last_node_block_hash = node_block_hash;
        records.push(record);
    }

    if !records.is_empty() {
        if let Err(err) = run_execution_bridge_retention_maintenance(
            execution_records_dir,
            execution_store,
            EXECUTION_BRIDGE_DEFAULT_HOT_WINDOW_HEIGHTS,
        ) {
            eprintln!(
                "execution bridge retention pin-set sync failed after height {}: {}",
                target_height, err
            );
        }
    }

    Ok(records)
}

fn normalize_execution_bridge_record_for_persist(
    record: &ExecutionBridgeRecord,
) -> ExecutionBridgeRecord {
    let mut normalized = record.clone();
    normalized.schema_version = EXECUTION_BRIDGE_RECORD_SCHEMA_V2;
    if normalized.latest_state_ref.is_none() {
        normalized.latest_state_ref = normalized.snapshot_ref.clone();
    }
    normalized
}

fn persist_execution_bridge_record_only(
    execution_records_dir: &Path,
    record: &ExecutionBridgeRecord,
) -> Result<Vec<u8>, String> {
    let normalized = normalize_execution_bridge_record_for_persist(record);
    let bytes = serde_json::to_vec_pretty(&normalized)
        .map_err(|err| format!("serialize execution bridge record failed: {}", err))?;
    let path = execution_bridge_record_path(execution_records_dir, normalized.height);
    super::write_bytes_atomic(path.as_path(), bytes.as_slice())?;
    Ok(bytes)
}

fn persist_execution_bridge_record(
    execution_records_dir: &Path,
    record: &ExecutionBridgeRecord,
) -> Result<(), String> {
    let bytes = persist_execution_bridge_record_only(execution_records_dir, record)?;
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
    use agent_world::runtime::{
        Action as RuntimeAction, ModuleArtifactIdentity, ModuleKind, ModuleLimits, ModuleManifest,
        ModuleRole, ModuleSubscription, ModuleSubscriptionStage,
    };
    use agent_world::simulator::{Action as SimulatorAction, ActionSubmitter};
    use agent_world_node::{NodeConsensusSnapshot, NodeRole};
    use agent_world_wasm_abi::{ModuleCallFailure, ModuleOutput};
    use agent_world_wasm_executor::FixedSandbox;
    use ed25519_dalek::{Signer, SigningKey};
    use sha2::{Digest, Sha256};
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_MODULE_ARTIFACT_SIGNER_NODE_ID: &str = "test.module.release.signer";

    fn signed_test_artifact_identity(wasm_hash: &str) -> ModuleArtifactIdentity {
        let source_hash = sha256_hex(format!("test-src:{wasm_hash}").as_bytes());
        let build_manifest_hash = sha256_hex(b"test-build-manifest-v1");
        let payload = ModuleArtifactIdentity::signing_payload_v1(
            wasm_hash,
            source_hash.as_str(),
            build_manifest_hash.as_str(),
            TEST_MODULE_ARTIFACT_SIGNER_NODE_ID,
        );
        let signing_key = test_module_artifact_signing_key();
        let signature = signing_key.sign(payload.as_slice());
        ModuleArtifactIdentity {
            source_hash,
            build_manifest_hash,
            signer_node_id: TEST_MODULE_ARTIFACT_SIGNER_NODE_ID.to_string(),
            signature_scheme: ModuleArtifactIdentity::SIGNATURE_SCHEME_ED25519.to_string(),
            artifact_signature: format!(
                "{}{}",
                ModuleArtifactIdentity::SIGNATURE_PREFIX_ED25519_V1,
                hex::encode(signature.to_bytes())
            ),
        }
    }

    fn test_module_artifact_signing_key() -> SigningKey {
        let seed_bytes = sha256_bytes(b"agent-world-test-module-artifact-signer-v1");
        SigningKey::from_bytes(&seed_bytes)
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        hex::encode(sha256_bytes(bytes))
    }

    fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
    }

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
        let mut sandbox = FixedSandbox::succeed(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: None,
            output_bytes: 0,
        });
        let mut state = ExecutionBridgeState::default();
        let records_dir = dir.join("records");

        let snapshot = sample_snapshot(2, Some("node-h2"));
        let records = bridge_committed_heights(
            &snapshot,
            1_000,
            &mut world,
            &mut sandbox,
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

        let latest_bytes = fs::read(records_dir.join("latest.json")).expect("read latest record");
        let latest_record: ExecutionBridgeRecord =
            serde_json::from_slice(latest_bytes.as_slice()).expect("parse latest record");
        assert_eq!(
            latest_record.schema_version,
            EXECUTION_BRIDGE_RECORD_SCHEMA_V2
        );
        assert_eq!(
            latest_record.latest_state_ref.as_deref(),
            latest_record.snapshot_ref.as_deref()
        );
        assert!(latest_record.commit_log_ref.is_none());
        assert!(latest_record.checkpoint_ref.is_none());
        assert!(latest_record.external_effect_ref.is_none());

        let latest_json: serde_json::Value =
            serde_json::from_slice(latest_bytes.as_slice()).expect("parse latest json");
        assert!(latest_json.get("schema_version").is_some());
        assert!(latest_json.get("latest_state_ref").is_some());
        assert!(latest_json.get("commit_log_ref").is_none());
        assert!(latest_json.get("checkpoint_ref").is_none());
        assert!(latest_json.get("external_effect_ref").is_none());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bridge_committed_heights_is_noop_when_height_not_advanced() {
        let dir = temp_dir("execution-bridge-noop");
        let store = LocalCasStore::new(dir.join("store"));
        let mut world = RuntimeWorld::new();
        let mut sandbox = FixedSandbox::succeed(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: None,
            output_bytes: 0,
        });
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
            &mut sandbox,
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
    fn execution_bridge_record_legacy_payload_defaults_latest_state_ref() {
        let legacy = serde_json::json!({
            "world_id": "w1",
            "height": 7,
            "node_block_hash": "node-h7",
            "execution_block_hash": "exec-h7",
            "execution_state_root": "state-r7",
            "journal_len": 3,
            "snapshot_ref": "cas:snapshot-7",
            "journal_ref": "cas:journal-7",
            "timestamp_ms": 7000
        });
        let record: ExecutionBridgeRecord =
            serde_json::from_value(legacy).expect("parse legacy execution bridge record");

        assert_eq!(record.schema_version, EXECUTION_BRIDGE_RECORD_SCHEMA_V1);
        assert_eq!(record.latest_state_ref.as_deref(), Some("cas:snapshot-7"));
        assert_eq!(record.snapshot_ref.as_deref(), Some("cas:snapshot-7"));
        assert_eq!(record.journal_ref.as_deref(), Some("cas:journal-7"));
        assert!(record.commit_log_ref.is_none());
        assert!(record.checkpoint_ref.is_none());
        assert!(record.external_effect_ref.is_none());
    }

    #[test]
    fn persist_execution_bridge_record_only_migrates_legacy_record_to_v2() {
        let dir = temp_dir("execution-bridge-legacy-migrate");
        let records_dir = dir.join("records");
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");
        let legacy = serde_json::json!({
            "world_id": "w1",
            "height": 7,
            "node_block_hash": "node-h7",
            "execution_block_hash": "exec-h7",
            "execution_state_root": "state-r7",
            "journal_len": 7,
            "snapshot_ref": "cas:snapshot-7",
            "journal_ref": "cas:journal-7",
            "timestamp_ms": 7000
        });
        let legacy_bytes = serde_json::to_vec_pretty(&legacy).expect("serialize legacy record");
        crate::write_bytes_atomic(
            execution_bridge_record_path(records_dir.as_path(), 7).as_path(),
            legacy_bytes.as_slice(),
        )
        .expect("persist legacy record");

        let record = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 7).as_path(),
        )
        .expect("load legacy record");
        assert_eq!(record.schema_version, EXECUTION_BRIDGE_RECORD_SCHEMA_V1);
        persist_execution_bridge_record_only(records_dir.as_path(), &record)
            .expect("rewrite legacy record as v2");

        let migrated = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 7).as_path(),
        )
        .expect("load migrated record");
        assert_eq!(migrated.schema_version, EXECUTION_BRIDGE_RECORD_SCHEMA_V2);
        assert_eq!(migrated.latest_state_ref.as_deref(), Some("cas:snapshot-7"));
        assert_eq!(migrated.snapshot_ref.as_deref(), Some("cas:snapshot-7"));
        assert_eq!(migrated.journal_ref.as_deref(), Some("cas:journal-7"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_bridge_retention_maintenance_skips_aggressive_sweep_for_legacy_records() {
        let dir = temp_dir("execution-bridge-legacy-safe-mode");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");

        let snapshot_ref = store
            .put_bytes(b"legacy-snapshot")
            .expect("store legacy snapshot");
        let journal_ref = store
            .put_bytes(b"legacy-journal")
            .expect("store legacy journal");
        let legacy = serde_json::json!({
            "world_id": "w1",
            "height": 1,
            "node_block_hash": "node-h1",
            "execution_block_hash": "exec-h1",
            "execution_state_root": "state-r1",
            "journal_len": 1,
            "snapshot_ref": snapshot_ref,
            "journal_ref": journal_ref,
            "timestamp_ms": 1000
        });
        let legacy_bytes = serde_json::to_vec_pretty(&legacy).expect("serialize legacy record");
        crate::write_bytes_atomic(
            execution_bridge_record_path(records_dir.as_path(), 1).as_path(),
            legacy_bytes.as_slice(),
        )
        .expect("persist legacy record");
        crate::write_bytes_atomic(
            records_dir.join("latest.json").as_path(),
            legacy_bytes.as_slice(),
        )
        .expect("persist legacy latest pointer");

        let freed_bytes =
            run_execution_bridge_retention_maintenance(records_dir.as_path(), &store, 1)
                .expect("run retention maintenance");
        assert_eq!(freed_bytes, 0);
        let record = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 1).as_path(),
        )
        .expect("load legacy record after maintenance");
        assert_eq!(record.schema_version, EXECUTION_BRIDGE_RECORD_SCHEMA_V1);
        assert!(record.snapshot_ref.is_some());
        assert!(record.journal_ref.is_some());
        assert!(store
            .has(record.snapshot_ref.as_deref().expect("legacy snapshot ref"))
            .expect("legacy snapshot still exists"));
        assert!(store
            .has(record.journal_ref.as_deref().expect("legacy journal ref"))
            .expect("legacy journal still exists"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_checkpoint_manifest_roundtrip_updates_latest_pointer() {
        let dir = temp_dir("execution-checkpoint-manifest");
        let records_dir = dir.join("records");
        let manifest = ExecutionCheckpointManifest::new(
            "w1".to_string(),
            12,
            "exec-h12".to_string(),
            "state-r12".to_string(),
            "cas:snapshot-12".to_string(),
            Some("cas:snapshot-12".to_string()),
            Some("cas:journal-12".to_string()),
            12_000,
        )
        .expect("build manifest");

        persist_execution_checkpoint_manifest(records_dir.as_path(), &manifest)
            .expect("persist manifest");
        let loaded = load_execution_checkpoint_manifest(
            execution_checkpoint_manifest_path(records_dir.as_path(), 12).as_path(),
        )
        .expect("load manifest");
        let latest = load_latest_execution_checkpoint_manifest(records_dir.as_path())
            .expect("load latest manifest")
            .expect("latest manifest should exist");

        assert_eq!(loaded, manifest);
        assert_eq!(latest, manifest);
        assert_eq!(
            latest.pinned_refs,
            vec!["cas:journal-12".to_string(), "cas:snapshot-12".to_string()]
        );
        assert!(execution_checkpoint_latest_path(records_dir.as_path()).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_checkpoint_manifest_rejects_tampered_latest_pointer() {
        let dir = temp_dir("execution-checkpoint-manifest-tamper");
        let records_dir = dir.join("records");
        let manifest = ExecutionCheckpointManifest::new(
            "w1".to_string(),
            4,
            "exec-h4".to_string(),
            "state-r4".to_string(),
            "cas:snapshot-4".to_string(),
            Some("cas:snapshot-4".to_string()),
            Some("cas:journal-4".to_string()),
            4_000,
        )
        .expect("build manifest");
        persist_execution_checkpoint_manifest(records_dir.as_path(), &manifest)
            .expect("persist manifest");

        let latest_path = execution_checkpoint_latest_path(records_dir.as_path());
        let mut latest_json: serde_json::Value = serde_json::from_slice(
            fs::read(latest_path.as_path())
                .expect("read latest pointer")
                .as_slice(),
        )
        .expect("parse latest pointer");
        latest_json["manifest_hash"] = serde_json::Value::String("tampered".to_string());
        let latest_bytes =
            serde_json::to_vec_pretty(&latest_json).expect("serialize tampered latest pointer");
        crate::write_bytes_atomic(latest_path.as_path(), latest_bytes.as_slice())
            .expect("persist tampered latest pointer");

        let err = load_latest_execution_checkpoint_manifest(records_dir.as_path())
            .expect_err("tampered latest pointer should fail");
        assert!(
            err.contains("hash mismatch"),
            "unexpected latest pointer error: {err}"
        );

        let _ = fs::remove_dir_all(dir);
    }

    fn persist_test_execution_record(
        records_dir: &Path,
        height: u64,
        block_hash: &str,
    ) -> ExecutionBridgeRecord {
        let record = ExecutionBridgeRecord::new_v2(
            "w1".to_string(),
            height,
            Some(format!("node-h{height}")),
            block_hash.to_string(),
            format!("state-r{height}"),
            height as usize,
            format!("cas:snapshot-{height}"),
            format!("cas:journal-{height}"),
            None,
            None,
            height as i64 * 1_000,
        );
        persist_execution_bridge_record(records_dir, &record)
            .expect("persist test execution record");
        record
    }

    fn persist_test_execution_record_with_store_refs(
        records_dir: &Path,
        store: &LocalCasStore,
        height: u64,
    ) -> ExecutionBridgeRecord {
        let snapshot_ref = store
            .put_bytes(format!("record-snapshot-{height}").as_bytes())
            .expect("store snapshot");
        let journal_ref = store
            .put_bytes(format!("record-journal-{height}").as_bytes())
            .expect("store journal");
        let simulator_snapshot_ref = store
            .put_bytes(format!("simulator-snapshot-{height}").as_bytes())
            .expect("store simulator snapshot");
        let simulator_journal_ref = store
            .put_bytes(format!("simulator-journal-{height}").as_bytes())
            .expect("store simulator journal");
        let external_effect_ref =
            persist_test_external_effect(store, "w1", height, format!("node-h{height}").as_str());
        let record = ExecutionBridgeRecord {
            latest_state_ref: Some(snapshot_ref.clone()),
            snapshot_ref: Some(snapshot_ref),
            journal_ref: Some(journal_ref),
            external_effect_ref: Some(external_effect_ref),
            simulator_mirror: Some(ExecutionSimulatorMirrorRecord {
                action_count: height as usize,
                rejected_action_count: 0,
                journal_len: height as usize,
                snapshot_ref: simulator_snapshot_ref,
                journal_ref: simulator_journal_ref,
                state_root: format!("simulator-state-{height}"),
            }),
            ..ExecutionBridgeRecord::new_v2(
                "w1".to_string(),
                height,
                Some(format!("node-h{height}")),
                format!("exec-h{height}"),
                format!("state-root-{height}"),
                height as usize,
                "placeholder-snapshot".to_string(),
                "placeholder-journal".to_string(),
                None,
                None,
                height as i64 * 1_000,
            )
        };
        persist_execution_bridge_record(records_dir, &record)
            .expect("persist test execution record with store refs");
        record
    }

    fn persist_test_external_effect(
        store: &LocalCasStore,
        world_id: &str,
        height: u64,
        node_block_hash: &str,
    ) -> String {
        let materialization = ExecutionExternalEffectMaterialization {
            schema_version: EXECUTION_EXTERNAL_EFFECT_SCHEMA_V1,
            contract: EXECUTION_EXTERNAL_EFFECT_CONTRACT_CLOSED_WORLD_V1.to_string(),
            world_id: world_id.to_string(),
            node_id: "node-a".to_string(),
            height,
            slot: height.saturating_sub(1),
            epoch: 0,
            node_block_hash: node_block_hash.to_string(),
            action_root: format!("action-root-{height}"),
            committed_at_unix_ms: height as i64 * 1_000,
            pre_step_execution_state_root: format!("pre-step-state-{height}"),
            world_manifest_hash: format!("manifest-hash-{height}"),
            active_modules_hash: execution_module_anchor_hash(&[]).expect("empty module hash"),
            committed_actions_hash: execution_committed_actions_hash(&[])
                .expect("empty actions hash"),
            active_modules: Vec::new(),
            committed_actions: Vec::new(),
            unresolved_inputs: Vec::new(),
        };
        persist_execution_external_effect_materialization(store, &materialization)
            .expect("persist test external effect")
    }

    #[test]
    fn execution_replay_plan_without_checkpoint_replays_full_log() {
        let dir = temp_dir("execution-replay-plan-full-log");
        let records_dir = dir.join("records");
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");
        persist_test_execution_record(records_dir.as_path(), 1, "exec-h1");
        persist_test_execution_record(records_dir.as_path(), 2, "exec-h2");
        persist_test_execution_record(records_dir.as_path(), 3, "exec-h3");

        let store = LocalCasStore::new(dir.join("store"));
        let plan = build_execution_replay_plan(records_dir.as_path(), &store, 3)
            .expect("build replay plan");
        assert_eq!(plan.target_height, 3);
        assert_eq!(plan.start_height, 1);
        assert!(plan.checkpoint.is_none());
        assert_eq!(plan.records.len(), 3);
        assert_eq!(plan.records[0].record.height, 1);
        assert_eq!(plan.records[2].record.height, 3);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_replay_plan_prefers_nearest_checkpoint_not_ahead_of_target() {
        let dir = temp_dir("execution-replay-plan-checkpoint");
        let records_dir = dir.join("records");
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");
        for height in 1..=6 {
            persist_test_execution_record(
                records_dir.as_path(),
                height,
                &format!("exec-h{height}"),
            );
        }
        let checkpoint_3 = ExecutionCheckpointManifest::new(
            "w1".to_string(),
            3,
            "exec-h3".to_string(),
            "state-r3".to_string(),
            "cas:snapshot-3".to_string(),
            Some("cas:snapshot-3".to_string()),
            Some("cas:journal-3".to_string()),
            3_000,
        )
        .expect("checkpoint 3");
        let checkpoint_5 = ExecutionCheckpointManifest::new(
            "w1".to_string(),
            5,
            "exec-h5".to_string(),
            "state-r5".to_string(),
            "cas:snapshot-5".to_string(),
            Some("cas:snapshot-5".to_string()),
            Some("cas:journal-5".to_string()),
            5_000,
        )
        .expect("checkpoint 5");
        persist_execution_checkpoint_manifest(records_dir.as_path(), &checkpoint_3)
            .expect("persist checkpoint 3");
        persist_execution_checkpoint_manifest(records_dir.as_path(), &checkpoint_5)
            .expect("persist checkpoint 5");

        let store = LocalCasStore::new(dir.join("store"));
        let plan = build_execution_replay_plan(records_dir.as_path(), &store, 6)
            .expect("build replay plan");
        assert_eq!(plan.start_height, 6);
        assert_eq!(plan.records.len(), 1);
        assert_eq!(plan.records[0].record.height, 6);
        assert_eq!(
            plan.checkpoint.as_ref().map(|manifest| manifest.height),
            Some(5)
        );

        let earlier_plan = build_execution_replay_plan(records_dir.as_path(), &store, 4)
            .expect("build earlier replay plan");
        assert_eq!(earlier_plan.start_height, 4);
        assert_eq!(earlier_plan.records.len(), 1);
        assert_eq!(earlier_plan.records[0].record.height, 4);
        assert_eq!(
            earlier_plan
                .checkpoint
                .as_ref()
                .map(|manifest| manifest.height),
            Some(3)
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_replay_plan_fails_closed_when_external_effect_blob_missing() {
        let dir = temp_dir("execution-replay-plan-missing-external-effect");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");
        let mut record = persist_test_execution_record(records_dir.as_path(), 1, "exec-h1");
        record.external_effect_ref = Some("missing-external-effect".to_string());
        persist_execution_bridge_record(records_dir.as_path(), &record)
            .expect("persist updated execution record");

        let err = build_execution_replay_plan(records_dir.as_path(), &store, 1)
            .expect_err("missing external effect blob should fail closed");
        assert!(
            err.contains("execution external effect CAS get failed"),
            "unexpected replay plan error: {err}"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_replay_plan_fails_closed_when_external_effect_mismatches_record() {
        let dir = temp_dir("execution-replay-plan-external-effect-mismatch");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");
        let mut record = persist_test_execution_record(records_dir.as_path(), 1, "exec-h1");
        let external_effect_ref = persist_test_external_effect(&store, "w2", 1, "node-h1");
        record.external_effect_ref = Some(external_effect_ref);
        persist_execution_bridge_record(records_dir.as_path(), &record)
            .expect("persist updated execution record");

        let err = build_execution_replay_plan(records_dir.as_path(), &store, 1)
            .expect_err("mismatched external effect should fail closed");
        assert!(
            err.contains("world_id mismatch"),
            "unexpected replay plan mismatch error: {err}"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_replay_plan_fails_closed_when_latest_checkpoint_pointer_is_corrupted() {
        let dir = temp_dir("execution-replay-plan-corrupted-checkpoint");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");
        persist_test_execution_record(records_dir.as_path(), 1, "exec-h1");
        persist_test_execution_record(records_dir.as_path(), 2, "exec-h2");
        persist_test_execution_record(records_dir.as_path(), 3, "exec-h3");
        let checkpoint = ExecutionCheckpointManifest::new(
            "w1".to_string(),
            2,
            "exec-h2".to_string(),
            "state-r2".to_string(),
            "cas:snapshot-2".to_string(),
            Some("cas:snapshot-2".to_string()),
            Some("cas:journal-2".to_string()),
            2_000,
        )
        .expect("checkpoint");
        persist_execution_checkpoint_manifest(records_dir.as_path(), &checkpoint)
            .expect("persist checkpoint");
        let latest_path = execution_checkpoint_latest_path(records_dir.as_path());
        let mut latest_json: serde_json::Value = serde_json::from_slice(
            fs::read(latest_path.as_path())
                .expect("read latest pointer")
                .as_slice(),
        )
        .expect("parse latest pointer");
        latest_json["manifest_hash"] = serde_json::Value::String("tampered".to_string());
        let latest_bytes =
            serde_json::to_vec_pretty(&latest_json).expect("serialize tampered latest pointer");
        crate::write_bytes_atomic(latest_path.as_path(), latest_bytes.as_slice())
            .expect("persist tampered latest pointer");

        let err = build_execution_replay_plan(records_dir.as_path(), &store, 3)
            .expect_err("corrupted checkpoint pointer should fail closed");
        assert!(
            err.contains("hash mismatch"),
            "unexpected checkpoint corruption error: {err}"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_checkpoint_cadence_trims_old_manifests_and_clears_record_refs() {
        let dir = temp_dir("execution-checkpoint-cadence-trim");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");

        for height in 1..=6 {
            let mut record = persist_test_execution_record_with_store_refs(
                records_dir.as_path(),
                &store,
                height,
            );
            record.checkpoint_ref =
                maybe_persist_execution_checkpoint_for_record(records_dir.as_path(), &record, 2, 2)
                    .expect("maybe persist checkpoint");
            persist_execution_bridge_record(records_dir.as_path(), &record)
                .expect("persist checkpointed record");
        }

        assert_eq!(
            list_execution_checkpoint_heights(records_dir.as_path())
                .expect("list checkpoint heights"),
            vec![4, 6]
        );
        let record_2 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 2).as_path(),
        )
        .expect("load record 2");
        let record_4 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 4).as_path(),
        )
        .expect("load record 4");
        let record_6 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 6).as_path(),
        )
        .expect("load record 6");
        assert!(record_2.checkpoint_ref.is_none());
        assert_eq!(
            record_4.checkpoint_ref.as_deref(),
            Some(execution_checkpoint_manifest_rel_path(4).as_str())
        );
        assert_eq!(
            record_6.checkpoint_ref.as_deref(),
            Some(execution_checkpoint_manifest_rel_path(6).as_str())
        );
        let latest = load_latest_execution_checkpoint_manifest(records_dir.as_path())
            .expect("load latest checkpoint")
            .expect("latest checkpoint exists");
        assert_eq!(latest.height, 6);

        let plan = build_execution_replay_plan(records_dir.as_path(), &store, 5)
            .expect("build replay plan from sparse checkpoint");
        assert_eq!(
            plan.checkpoint.as_ref().map(|manifest| manifest.height),
            Some(4)
        );
        assert_eq!(plan.start_height, 5);
        assert_eq!(plan.records.len(), 1);
        assert_eq!(plan.records[0].record.height, 5);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bridge_committed_heights_persists_sparse_checkpoint_at_default_interval() {
        let dir = temp_dir("execution-bridge-default-checkpoint");
        let store = LocalCasStore::new(dir.join("store"));
        let mut world = RuntimeWorld::new();
        let mut sandbox = FixedSandbox::succeed(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: None,
            output_bytes: 0,
        });
        let mut state = ExecutionBridgeState::default();
        let records_dir = dir.join("records");
        let snapshot = sample_snapshot(
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS,
            Some("node-h32"),
        );

        let records = bridge_committed_heights(
            &snapshot,
            1_000,
            &mut world,
            &mut sandbox,
            &store,
            records_dir.as_path(),
            &mut state,
        )
        .expect("bridge committed heights");

        assert_eq!(
            records.len() as u64,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS
        );
        let latest_record = records.last().expect("latest record");
        assert_eq!(
            latest_record.checkpoint_ref.as_deref(),
            Some(
                execution_checkpoint_manifest_rel_path(
                    EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS,
                )
                .as_str()
            )
        );
        let latest_checkpoint = load_latest_execution_checkpoint_manifest(records_dir.as_path())
            .expect("load latest checkpoint")
            .expect("latest checkpoint exists");
        assert_eq!(
            latest_checkpoint.height,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS
        );

        let plan = build_execution_replay_plan(
            records_dir.as_path(),
            &store,
            EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS,
        )
        .expect("build replay plan");
        assert_eq!(
            plan.checkpoint.as_ref().map(|manifest| manifest.height),
            Some(EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS)
        );
        assert!(plan.records.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_bridge_retention_maintenance_clears_archive_refs_and_prunes_orphans() {
        let dir = temp_dir("execution-bridge-retention-maintenance");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");

        let mut records = Vec::new();
        for height in 1..=6 {
            let mut record = persist_test_execution_record_with_store_refs(
                records_dir.as_path(),
                &store,
                height,
            );
            record.checkpoint_ref =
                maybe_persist_execution_checkpoint_for_record(records_dir.as_path(), &record, 2, 2)
                    .expect("maybe persist checkpoint");
            persist_execution_bridge_record(records_dir.as_path(), &record)
                .expect("persist checkpointed record");
            records.push(record);
        }

        let freed_bytes =
            run_execution_bridge_retention_maintenance(records_dir.as_path(), &store, 2)
                .expect("run retention maintenance");
        assert!(freed_bytes > 0, "expected orphan sweep to free bytes");

        let record_1 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 1).as_path(),
        )
        .expect("load record 1");
        let record_4 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 4).as_path(),
        )
        .expect("load record 4");
        let record_5 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 5).as_path(),
        )
        .expect("load record 5");
        let record_6 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 6).as_path(),
        )
        .expect("load record 6");

        assert!(record_1.latest_state_ref.is_none());
        assert!(record_1.snapshot_ref.is_none());
        assert!(record_1.journal_ref.is_none());
        assert!(record_1.simulator_mirror.is_none());
        assert_eq!(
            record_4.checkpoint_ref.as_deref(),
            Some(execution_checkpoint_manifest_rel_path(4).as_str())
        );
        assert!(record_4.snapshot_ref.is_none());
        assert!(record_4.journal_ref.is_none());
        assert!(record_4.simulator_mirror.is_none());
        assert!(record_5.snapshot_ref.is_some());
        assert!(record_5.journal_ref.is_some());
        assert!(record_6.snapshot_ref.is_some());
        assert!(record_6.journal_ref.is_some());

        assert!(!store
            .has(
                records[0]
                    .snapshot_ref
                    .as_deref()
                    .expect("record1 snapshot ref")
            )
            .expect("check archive snapshot"));
        assert!(store
            .has(
                records[3]
                    .snapshot_ref
                    .as_deref()
                    .expect("record4 snapshot ref")
            )
            .expect("check checkpoint snapshot"));
        assert!(store
            .has(
                records[4]
                    .journal_ref
                    .as_deref()
                    .expect("record5 journal ref")
            )
            .expect("check hot journal"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bridge_committed_heights_sweeps_archive_refs_outside_default_hot_window() {
        let dir = temp_dir("execution-bridge-default-retention-sweep");
        let store = LocalCasStore::new(dir.join("store"));
        let mut world = RuntimeWorld::new();
        let mut sandbox = FixedSandbox::succeed(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            tick_lifecycle: None,
            output_bytes: 0,
        });
        let mut state = ExecutionBridgeState::default();
        let records_dir = dir.join("records");
        let target_height = EXECUTION_BRIDGE_DEFAULT_HOT_WINDOW_HEIGHTS
            + EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS;
        let snapshot = sample_snapshot(target_height, Some("node-h64"));

        let records = bridge_committed_heights(
            &snapshot,
            1_000,
            &mut world,
            &mut sandbox,
            &store,
            records_dir.as_path(),
            &mut state,
        )
        .expect("bridge committed heights");

        let record_1 = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), 1).as_path(),
        )
        .expect("load record 1");
        let checkpoint_height = EXECUTION_BRIDGE_DEFAULT_CHECKPOINT_INTERVAL_HEIGHTS;
        let record_checkpoint = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), checkpoint_height).as_path(),
        )
        .expect("load checkpoint record");
        let record_hot = load_execution_bridge_record(
            execution_bridge_record_path(records_dir.as_path(), checkpoint_height + 1).as_path(),
        )
        .expect("load hot record");

        assert!(record_1.snapshot_ref.is_none());
        assert!(record_1.journal_ref.is_none());
        assert_eq!(
            record_checkpoint.checkpoint_ref.as_deref(),
            Some(execution_checkpoint_manifest_rel_path(checkpoint_height).as_str())
        );
        assert!(record_checkpoint.snapshot_ref.is_none());
        assert!(record_checkpoint.journal_ref.is_none());
        assert!(record_hot.snapshot_ref.is_some());
        assert!(record_hot.journal_ref.is_some());

        assert!(!store
            .has(
                records[0]
                    .snapshot_ref
                    .as_deref()
                    .expect("record1 snapshot ref")
            )
            .expect("check archive snapshot"));
        let checkpoint_index = checkpoint_height.saturating_sub(1) as usize;
        assert!(store
            .has(
                records[checkpoint_index]
                    .snapshot_ref
                    .as_deref()
                    .expect("checkpoint snapshot ref"),
            )
            .expect("check checkpoint snapshot"));
        assert!(store
            .has(
                records[checkpoint_index + 1]
                    .journal_ref
                    .as_deref()
                    .expect("hot journal ref"),
            )
            .expect("check hot journal"));

        let plan =
            build_execution_replay_plan(records_dir.as_path(), &store, checkpoint_height + 8)
                .expect("build replay plan from sparse checkpoint");
        assert_eq!(
            plan.checkpoint.as_ref().map(|manifest| manifest.height),
            Some(checkpoint_height)
        );
        assert_eq!(plan.start_height, checkpoint_height + 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_bridge_pin_set_keeps_latest_head_and_hot_window_refs() {
        let dir = temp_dir("execution-bridge-pin-set-hot-window");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");

        let mut all_refs = Vec::new();
        let mut records = Vec::new();
        for height in 1..=4 {
            let record = persist_test_execution_record_with_store_refs(
                records_dir.as_path(),
                &store,
                height,
            );
            all_refs.extend(record.snapshot_ref.iter().cloned());
            all_refs.extend(record.journal_ref.iter().cloned());
            all_refs.extend(record.latest_state_ref.iter().cloned());
            all_refs.extend(record.external_effect_ref.iter().cloned());
            if let Some(simulator_mirror) = record.simulator_mirror.as_ref() {
                all_refs.push(simulator_mirror.snapshot_ref.clone());
                all_refs.push(simulator_mirror.journal_ref.clone());
            }
            records.push(record);
        }
        all_refs.sort();
        all_refs.dedup();

        for content_ref in &all_refs {
            store.pin(content_ref.as_str()).expect("pre-pin record ref");
        }

        let pin_set = sync_execution_bridge_pin_set(records_dir.as_path(), &store, 2)
            .expect("sync execution bridge pin set");
        assert_eq!(pin_set.latest_height, Some(4));
        assert_eq!(pin_set.hot_window_start_height, Some(3));

        let actual_pins = store
            .list_pins()
            .expect("list pins")
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut expected_pins = BTreeSet::new();
        for record in &records {
            expected_pins.extend(record.external_effect_ref.iter().cloned());
            if record.height >= 3 {
                expected_pins.extend(record.snapshot_ref.iter().cloned());
                expected_pins.extend(record.journal_ref.iter().cloned());
                if let Some(simulator_mirror) = record.simulator_mirror.as_ref() {
                    expected_pins.insert(simulator_mirror.snapshot_ref.clone());
                    expected_pins.insert(simulator_mirror.journal_ref.clone());
                }
            }
            if record.height == 4 {
                expected_pins.extend(record.latest_state_ref.iter().cloned());
            }
        }
        assert_eq!(actual_pins, expected_pins);
        assert!(!records[0]
            .snapshot_ref
            .as_ref()
            .is_some_and(|snapshot_ref| actual_pins.contains(snapshot_ref)));
        assert!(!records[1]
            .journal_ref
            .as_ref()
            .is_some_and(|journal_ref| actual_pins.contains(journal_ref)));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_bridge_pin_set_keeps_checkpoint_refs_outside_hot_window() {
        let dir = temp_dir("execution-bridge-pin-set-checkpoint");
        let records_dir = dir.join("records");
        let store = LocalCasStore::new(dir.join("store"));
        fs::create_dir_all(records_dir.as_path()).expect("create records dir");

        for height in 1..=3 {
            let _ = persist_test_execution_record_with_store_refs(
                records_dir.as_path(),
                &store,
                height,
            );
        }

        let checkpoint_latest_state_ref = store
            .put_bytes(b"checkpoint-latest-state")
            .expect("store checkpoint latest state");
        let checkpoint_snapshot_ref = store
            .put_bytes(b"checkpoint-snapshot")
            .expect("store checkpoint snapshot");
        let checkpoint_journal_ref = store
            .put_bytes(b"checkpoint-journal")
            .expect("store checkpoint journal");
        let checkpoint = ExecutionCheckpointManifest::new(
            "w1".to_string(),
            1,
            "exec-h1".to_string(),
            "state-root-1".to_string(),
            checkpoint_latest_state_ref.clone(),
            Some(checkpoint_snapshot_ref.clone()),
            Some(checkpoint_journal_ref.clone()),
            1_000,
        )
        .expect("checkpoint");
        persist_execution_checkpoint_manifest(records_dir.as_path(), &checkpoint)
            .expect("persist checkpoint");

        let pin_set = sync_execution_bridge_pin_set(records_dir.as_path(), &store, 1)
            .expect("sync execution bridge pin set");
        let actual_pins = pin_set.pinned_refs;
        assert!(actual_pins.contains(&checkpoint_latest_state_ref));
        assert!(actual_pins.contains(&checkpoint_snapshot_ref));
        assert!(actual_pins.contains(&checkpoint_journal_ref));

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

    fn tick_manifest(wasm_hash: &str) -> ModuleManifest {
        ModuleManifest {
            module_id: "m.test.tick".to_string(),
            name: "Tick Test".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Reducer,
            role: ModuleRole::Rule,
            wasm_hash: wasm_hash.to_string(),
            interface_version: "wasm-1".to_string(),
            abi_contract: agent_world_wasm_abi::ModuleAbiContract::default(),
            exports: vec!["reduce".to_string()],
            subscriptions: vec![ModuleSubscription {
                event_kinds: Vec::new(),
                action_kinds: Vec::new(),
                stage: Some(ModuleSubscriptionStage::Tick),
                filters: None,
            }],
            required_caps: Vec::new(),
            artifact_identity: Some(signed_test_artifact_identity(wasm_hash)),
            limits: ModuleLimits::default(),
        }
    }

    #[test]
    fn node_runtime_execution_driver_commit_routes_modules_via_step_with_modules() {
        let dir = temp_dir("execution-driver-modules");
        let state_path = dir.join("state.json");
        let world_dir = dir.join("world");
        let records_dir = dir.join("records");
        let storage_root = dir.join("store");

        let wasm_bytes = b"bridge-modules-wasm".to_vec();
        let wasm_hash = {
            let mut hasher = Sha256::new();
            hasher.update(wasm_bytes.as_slice());
            hex::encode(hasher.finalize())
        };
        let manifest = tick_manifest(&wasm_hash);
        let mut world = RuntimeWorld::new();
        world.submit_action(RuntimeAction::RegisterAgent {
            agent_id: "agent-0".to_string(),
            pos: agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        });
        world.step().expect("register");
        world
            .set_agent_resource_balance(
                "agent-0",
                agent_world::simulator::ResourceKind::Electricity,
                128,
            )
            .expect("seed electricity");
        world
            .set_agent_resource_balance("agent-0", agent_world::simulator::ResourceKind::Data, 64)
            .expect("seed data");
        world.submit_action(RuntimeAction::DeployModuleArtifact {
            publisher_agent_id: "agent-0".to_string(),
            wasm_hash: wasm_hash.clone(),
            wasm_bytes: wasm_bytes.clone(),
        });
        world.step().expect("deploy");
        world.submit_action(RuntimeAction::InstallModuleFromArtifact {
            installer_agent_id: "agent-0".to_string(),
            manifest: manifest.clone(),
            activate: true,
        });
        world.step().expect("install");

        let expected_trace = format!(
            "tick-{}-{}",
            world.state().time.saturating_add(1),
            manifest.module_id
        );
        let sandbox = FixedSandbox::fail(ModuleCallFailure {
            module_id: manifest.module_id.clone(),
            trace_id: expected_trace.clone(),
            code: agent_world_wasm_abi::ModuleCallErrorCode::PolicyDenied,
            detail: "forced failure for routing assertion".to_string(),
        });
        let mut driver = NodeRuntimeExecutionDriver::new_with_sandbox(
            state_path,
            world_dir,
            records_dir,
            storage_root,
            ExecutionBridgeState::default(),
            world,
            Box::new(sandbox.clone()),
        );

        let empty_action_root = compute_consensus_action_root(&[]).expect("empty action root");
        let err = driver
            .on_commit(NodeExecutionCommitContext {
                world_id: "w1".to_string(),
                node_id: "node-a".to_string(),
                height: 1,
                slot: 0,
                epoch: 0,
                node_block_hash: "node-h1".to_string(),
                action_root: empty_action_root,
                committed_actions: Vec::new(),
                committed_at_unix_ms: 1_000,
            })
            .expect_err("forced module failure should bubble");
        assert!(
            err.contains("world.step failed"),
            "unexpected error from commit path: {err}"
        );

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

        let store = LocalCasStore::new(dir.join("store"));
        let record_bytes = fs::read(records_dir.join("00000000000000000002.json"))
            .expect("read second execution bridge record");
        let record: ExecutionBridgeRecord =
            serde_json::from_slice(record_bytes.as_slice()).expect("parse second record");
        let external_effect_ref = record
            .external_effect_ref
            .as_deref()
            .expect("external effect ref should exist");
        let external_effect =
            load_execution_external_effect_materialization(&store, external_effect_ref)
                .expect("load external effect materialization");
        assert_eq!(external_effect.height, 2);
        assert_eq!(external_effect.slot, 1);
        assert_eq!(external_effect.epoch, 0);
        assert_eq!(
            external_effect.action_root,
            compute_consensus_action_root(&[]).expect("empty root")
        );
        assert!(external_effect.committed_actions.is_empty());
        assert!(external_effect.unresolved_inputs.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn node_runtime_execution_driver_tolerates_non_contiguous_commit_heights() {
        let dir = temp_dir("execution-driver-gap");
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
        let empty_action_root = compute_consensus_action_root(&[]).expect("empty action root");

        driver
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
        let gap_commit = driver
            .on_commit(NodeExecutionCommitContext {
                world_id: "w1".to_string(),
                node_id: "node-a".to_string(),
                height: 3,
                slot: 2,
                epoch: 0,
                node_block_hash: "node-h3".to_string(),
                action_root: empty_action_root,
                committed_actions: Vec::new(),
                committed_at_unix_ms: 3_000,
            })
            .expect("gap commit");

        assert_eq!(gap_commit.execution_height, 3);
        assert!(records_dir.join("00000000000000000001.json").exists());
        assert!(records_dir.join("00000000000000000003.json").exists());
        assert!(!records_dir.join("00000000000000000002.json").exists());

        let state = load_execution_bridge_state(state_path.as_path()).expect("load state");
        assert_eq!(state.last_applied_committed_height, 3);
        assert_eq!(state.last_node_block_hash.as_deref(), Some("node-h3"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn node_runtime_execution_driver_processes_simulator_payload_envelope() {
        let dir = temp_dir("execution-driver-simulator-payload");
        let state_path = dir.join("state.json");
        let world_dir = dir.join("world");
        let simulator_world_dir = simulator_world_dir_from_execution_world_dir(world_dir.as_path());
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
        let expected_action_root = action_root.clone();
        let expected_payload_hash = committed_action.payload_hash.clone();

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
        let record_bytes = fs::read(records_dir.join("00000000000000000001.json"))
            .expect("read execution bridge record");
        let record: ExecutionBridgeRecord =
            serde_json::from_slice(record_bytes.as_slice()).expect("parse execution bridge record");
        assert_eq!(record.schema_version, EXECUTION_BRIDGE_RECORD_SCHEMA_V2);
        assert_eq!(
            record.latest_state_ref.as_deref(),
            record.snapshot_ref.as_deref()
        );
        assert!(record
            .snapshot_ref
            .as_deref()
            .is_some_and(|snapshot_ref| !snapshot_ref.is_empty()));
        assert!(record
            .journal_ref
            .as_deref()
            .is_some_and(|journal_ref| !journal_ref.is_empty()));
        let external_effect_ref = record
            .external_effect_ref
            .as_deref()
            .expect("external effect ref should exist");
        let store = LocalCasStore::new(dir.join("store"));
        let external_effect =
            load_execution_external_effect_materialization(&store, external_effect_ref)
                .expect("load external effect materialization");
        assert_eq!(external_effect.height, 1);
        assert_eq!(external_effect.slot, 0);
        assert_eq!(external_effect.epoch, 0);
        assert_eq!(external_effect.action_root, expected_action_root);
        assert_eq!(external_effect.committed_actions.len(), 1);
        assert_eq!(external_effect.committed_actions[0].action_id, 1);
        assert_eq!(
            external_effect.committed_actions[0].payload_hash,
            expected_payload_hash
        );
        assert!(external_effect.unresolved_inputs.is_empty());
        let simulator = record
            .simulator_mirror
            .expect("simulator mirror record should exist");
        assert_eq!(simulator.action_count, 1);
        assert_eq!(simulator.rejected_action_count, 1);
        assert!(!simulator.snapshot_ref.is_empty());
        assert!(!simulator.journal_ref.is_empty());
        assert!(!simulator.state_root.is_empty());
        assert!(simulator_world_dir.join("snapshot.json").exists());
        assert!(simulator_world_dir.join("journal.json").exists());
        let _ = fs::remove_dir_all(dir);
    }
}
