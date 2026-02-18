use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::*;

#[derive(Clone)]
struct RecordingExecutionHook {
    calls: Arc<Mutex<Vec<NodeExecutionCommitContext>>>,
}

impl RecordingExecutionHook {
    fn new(calls: Arc<Mutex<Vec<NodeExecutionCommitContext>>>) -> Self {
        Self { calls }
    }
}

impl NodeExecutionHook for RecordingExecutionHook {
    fn on_commit(
        &mut self,
        context: NodeExecutionCommitContext,
    ) -> Result<NodeExecutionCommitResult, String> {
        self.calls
            .lock()
            .expect("lock execution calls")
            .push(context.clone());
        Ok(NodeExecutionCommitResult {
            execution_height: context.height,
            execution_block_hash: format!("exec-block-{:020}", context.height),
            execution_state_root: format!("exec-state-{:020}", context.height),
        })
    }
}

#[test]
fn runtime_execution_hook_receives_sorted_committed_actions() {
    let config = NodeConfig::new("node-action", "world-action", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick interval");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let hook = RecordingExecutionHook::new(Arc::clone(&calls));
    let mut runtime = NodeRuntime::new(config).with_execution_hook(hook);

    let payload_b = serde_cbor::to_vec(&serde_json::json!({"kind": "b"})).expect("payload b");
    let payload_a = serde_cbor::to_vec(&serde_json::json!({"kind": "a"})).expect("payload a");
    runtime
        .submit_consensus_action_payload(2, payload_b)
        .expect("submit action b");
    runtime
        .submit_consensus_action_payload(1, payload_a)
        .expect("submit action a");

    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(120));
    runtime.stop().expect("stop");

    let execution_calls = calls.lock().expect("lock calls");
    let with_actions = execution_calls
        .iter()
        .find(|call| !call.committed_actions.is_empty())
        .expect("at least one commit should carry actions");
    let ordered_ids: Vec<u64> = with_actions
        .committed_actions
        .iter()
        .map(|action| action.action_id)
        .collect();
    assert_eq!(ordered_ids, vec![1, 2]);
    let computed_root =
        compute_consensus_action_root(with_actions.committed_actions.as_slice()).expect("root");
    assert_eq!(computed_root, with_actions.action_root);
}

#[test]
fn submit_consensus_action_payload_rejects_zero_action_id() {
    let runtime = NodeRuntime::new(
        NodeConfig::new("node-action-id", "world-action-id", NodeRole::Observer).expect("config"),
    );
    let err = runtime
        .submit_consensus_action_payload(0, vec![0_u8])
        .expect_err("zero action id must fail");
    assert!(matches!(err, NodeError::Consensus { .. }));
}

#[test]
fn role_parse_roundtrip() {
    for role in [NodeRole::Sequencer, NodeRole::Storage, NodeRole::Observer] {
        let parsed = NodeRole::from_str(role.as_str()).expect("parse role");
        assert_eq!(parsed, role);
    }
}

#[test]
fn config_rejects_invalid_pos_config() {
    let result = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("base config")
        .with_pos_config(NodePosConfig::ethereum_like(vec![]));
    assert!(matches!(result, Err(NodeError::InvalidConfig { .. })));
}

#[test]
fn runtime_drains_committed_action_batches_for_viewer_consumers() {
    let config = NodeConfig::new("node-drain", "world-drain", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick interval");
    let mut runtime = NodeRuntime::new(config);

    let payload_b = serde_cbor::to_vec(&serde_json::json!({"kind": "b"})).expect("payload b");
    let payload_a = serde_cbor::to_vec(&serde_json::json!({"kind": "a"})).expect("payload a");
    runtime
        .submit_consensus_action_payload(2, payload_b)
        .expect("submit action b");
    runtime
        .submit_consensus_action_payload(1, payload_a)
        .expect("submit action a");

    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(120));
    runtime.stop().expect("stop");

    let batches = runtime.drain_committed_action_batches();
    assert!(!batches.is_empty());
    let with_actions = batches
        .iter()
        .find(|batch| !batch.actions.is_empty())
        .expect("at least one committed batch should carry actions");
    let ordered_ids: Vec<u64> = with_actions
        .actions
        .iter()
        .map(|action| action.action_id)
        .collect();
    assert_eq!(ordered_ids, vec![1, 2]);
}
