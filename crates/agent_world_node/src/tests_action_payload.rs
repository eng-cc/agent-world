use std::collections::BTreeMap;
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
fn runtime_dequeues_actions_with_engine_capacity_limit() {
    let config = NodeConfig::new("node-cap", "world-cap", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick interval")
        .with_max_pending_consensus_actions(16)
        .expect("runtime pending limit")
        .with_max_engine_pending_consensus_actions(1)
        .expect("engine pending limit");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let hook = RecordingExecutionHook::new(Arc::clone(&calls));
    let mut runtime = NodeRuntime::new(config).with_execution_hook(hook);

    for action_id in 1..=3 {
        runtime
            .submit_consensus_action_payload(action_id, vec![action_id as u8])
            .expect("submit action");
    }

    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(240));
    runtime.stop().expect("stop");

    let snapshot = runtime.snapshot();
    assert!(
        snapshot.last_error.is_none(),
        "engine should not saturate while runtime dequeues incrementally: {:?}",
        snapshot.last_error
    );

    let calls = calls.lock().expect("lock calls");
    let committed_ids = calls
        .iter()
        .flat_map(|call| call.committed_actions.iter().map(|action| action.action_id))
        .collect::<Vec<_>>();
    assert!(
        committed_ids.windows(2).all(|pair| pair[0] <= pair[1]),
        "committed action ids should remain monotonic: {committed_ids:?}"
    );
    assert!(
        committed_ids.iter().copied().eq([1, 2, 3]),
        "all queued actions should be eventually committed exactly once: {committed_ids:?}"
    );
}

#[test]
fn pos_engine_rejects_tick_when_engine_pending_limit_exceeded() {
    let config = NodeConfig::new(
        "node-engine-limit",
        "world-engine-limit",
        NodeRole::Observer,
    )
    .expect("config")
    .with_max_engine_pending_consensus_actions(1)
    .expect("engine pending limit");
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let action_1 = NodeConsensusAction::from_payload(1, config.player_id.clone(), vec![1_u8])
        .expect("action 1");
    let action_2 = NodeConsensusAction::from_payload(2, config.player_id.clone(), vec![2_u8])
        .expect("action 2");
    let err = engine
        .tick(
            &config.node_id,
            &config.world_id,
            1_000,
            None,
            None,
            None,
            None,
            vec![action_1, action_2],
            None,
        )
        .expect_err("engine should reject merged queue over capacity");
    assert!(matches!(err, NodeError::Consensus { .. }));
    assert!(
        err.to_string().contains("engine buffer saturated"),
        "unexpected error: {err}"
    );
}

#[test]
fn pos_engine_pending_capacity_reserves_rejected_proposal_actions() {
    let config = NodeConfig::new(
        "node-capacity-reserve",
        "world-capacity-reserve",
        NodeRole::Observer,
    )
    .expect("config")
    .with_max_engine_pending_consensus_actions(4)
    .expect("engine pending limit");
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let queued = NodeConsensusAction::from_payload(1, config.player_id.clone(), vec![1_u8])
        .expect("queued action");
    let reserved_a = NodeConsensusAction::from_payload(2, config.player_id.clone(), vec![2_u8])
        .expect("reserved action a");
    let reserved_b = NodeConsensusAction::from_payload(3, config.player_id.clone(), vec![3_u8])
        .expect("reserved action b");
    let action_root = compute_consensus_action_root(&[reserved_a.clone(), reserved_b.clone()])
        .expect("action root");

    engine
        .pending_consensus_actions
        .insert(queued.action_id, queued);
    engine.pending = Some(PendingProposal {
        height: 1,
        slot: 0,
        epoch: 0,
        proposer_id: config.node_id.clone(),
        block_hash: "pending-block".to_string(),
        action_root,
        committed_actions: vec![reserved_a, reserved_b],
        attestations: BTreeMap::new(),
        approved_stake: 0,
        rejected_stake: 0,
        status: PosConsensusStatus::Pending,
    });

    assert_eq!(
        engine.pending_consensus_action_capacity(),
        1,
        "capacity should reserve space for requeueing actions from the pending proposal"
    );
}

#[test]
fn pos_engine_apply_rejected_decision_surfaces_requeue_overflow_instead_of_dropping() {
    let config = NodeConfig::new(
        "node-requeue-overflow",
        "world-requeue-overflow",
        NodeRole::Observer,
    )
    .expect("config")
    .with_max_engine_pending_consensus_actions(2)
    .expect("engine pending limit");
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let queued = NodeConsensusAction::from_payload(1, config.player_id.clone(), vec![1_u8])
        .expect("queued action");
    engine
        .pending_consensus_actions
        .insert(queued.action_id, queued);

    let rejected_a = NodeConsensusAction::from_payload(2, config.player_id.clone(), vec![2_u8])
        .expect("rejected action a");
    let rejected_b = NodeConsensusAction::from_payload(3, config.player_id.clone(), vec![3_u8])
        .expect("rejected action b");
    let action_root = compute_consensus_action_root(&[rejected_a.clone(), rejected_b.clone()])
        .expect("action root");
    let decision = PosDecision {
        height: 7,
        slot: 6,
        epoch: 0,
        status: PosConsensusStatus::Rejected,
        block_hash: "rejected-block".to_string(),
        action_root,
        committed_actions: vec![rejected_a, rejected_b],
        approved_stake: 0,
        rejected_stake: 100,
        required_stake: 67,
        total_stake: 100,
    };

    let err = engine
        .apply_decision(&decision)
        .expect_err("requeue overflow must return an explicit error");
    let reason = err.to_string();
    assert!(
        reason.contains("requeue rejected consensus actions failed"),
        "error should describe requeue failure context: {reason}"
    );
    assert!(
        reason.contains("engine buffer saturated"),
        "error should preserve the saturation reason: {reason}"
    );
    assert_eq!(
        engine.pending_consensus_actions.len(),
        1,
        "existing queued actions should remain intact when requeue fails"
    );
    assert_eq!(
        engine.next_height, 1,
        "engine height should not advance when rejected actions cannot be requeued"
    );
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
fn submit_consensus_action_payload_as_player_rejects_player_mismatch() {
    let runtime = NodeRuntime::new(
        NodeConfig::new("node-action-id", "world-action-id", NodeRole::Observer).expect("config"),
    );
    let err = runtime
        .submit_consensus_action_payload_as_player("other-player", 1, vec![1_u8, 2, 3])
        .expect_err("mismatched player must fail");
    assert!(matches!(err, NodeError::Consensus { .. }));
}

#[test]
fn submit_consensus_action_payload_rejects_payload_over_limit() {
    let config = NodeConfig::new("node-limit", "world-limit", NodeRole::Observer)
        .expect("config")
        .with_max_consensus_action_payload_bytes(4)
        .expect("payload limit");
    let runtime = NodeRuntime::new(config);
    let err = runtime
        .submit_consensus_action_payload(1, vec![1_u8, 2, 3, 4, 5])
        .expect_err("oversized payload must fail");
    assert!(matches!(err, NodeError::Consensus { .. }));
    assert!(
        err.to_string().contains("payload too large"),
        "unexpected error: {err}"
    );
}

#[test]
fn submit_consensus_action_payload_rejects_queue_saturation() {
    let config = NodeConfig::new("node-queue", "world-queue", NodeRole::Observer)
        .expect("config")
        .with_max_pending_consensus_actions(1)
        .expect("queue limit");
    let runtime = NodeRuntime::new(config);
    runtime
        .submit_consensus_action_payload(1, vec![1_u8, 2, 3])
        .expect("first action should be accepted");
    let err = runtime
        .submit_consensus_action_payload(2, vec![4_u8, 5, 6])
        .expect_err("second action must fail after queue reaches limit");
    assert!(matches!(err, NodeError::Consensus { .. }));
    assert!(
        err.to_string().contains("queue saturated"),
        "unexpected error: {err}"
    );
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
fn config_accepts_extreme_supermajority_ratio_just_above_half() {
    let denominator = u64::MAX;
    let numerator = denominator / 2 + 1;
    let mut pos_config = NodePosConfig::ethereum_like(vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ]);
    pos_config.supermajority_numerator = numerator;
    pos_config.supermajority_denominator = denominator;

    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("base config")
        .with_pos_config(pos_config)
        .expect("extreme ratio should be valid");
    assert_eq!(config.pos_config.supermajority_numerator, numerator);
    assert_eq!(config.pos_config.supermajority_denominator, denominator);
}

#[test]
fn config_rejects_duplicate_validator_player_bindings() {
    let mut validator_player_ids = BTreeMap::new();
    validator_player_ids.insert("node-a".to_string(), "player-1".to_string());
    validator_player_ids.insert("node-b".to_string(), "player-1".to_string());
    let result = NodePosConfig::ethereum_like(vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ])
    .with_validator_player_ids(validator_player_ids);
    assert!(matches!(result, Err(NodeError::InvalidConfig { .. })));
}

#[test]
fn runtime_drains_committed_action_batches_for_viewer_consumers() {
    let config = NodeConfig::new("node-drain", "world-drain", NodeRole::Sequencer)
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

#[test]
fn runtime_committed_batches_respect_hot_window_limit() {
    let config = NodeConfig::new(
        "node-batch-window",
        "world-batch-window",
        NodeRole::Sequencer,
    )
    .expect("config")
    .with_tick_interval(Duration::from_millis(10))
    .expect("tick interval")
    .with_max_engine_pending_consensus_actions(1)
    .expect("engine pending limit")
    .with_max_committed_action_batches(2)
    .expect("batch window");
    let mut runtime = NodeRuntime::new(config).with_execution_hook(RecordingExecutionHook::new(
        Arc::new(Mutex::new(Vec::new())),
    ));
    for action_id in 1..=5 {
        runtime
            .submit_consensus_action_payload(action_id, vec![action_id as u8])
            .expect("submit action");
    }

    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(260));
    runtime.stop().expect("stop");

    let snapshot = runtime.snapshot();
    assert!(
        snapshot.consensus.committed_height >= 5,
        "expected >=5 committed heights, got {}",
        snapshot.consensus.committed_height
    );

    let batches = runtime.drain_committed_action_batches();
    assert_eq!(batches.len(), 2, "committed batch window must be capped");
    let retained_action_ids = batches
        .iter()
        .flat_map(|batch| batch.actions.iter().map(|action| action.action_id))
        .collect::<Vec<_>>();
    assert_eq!(retained_action_ids, vec![4, 5]);
}
