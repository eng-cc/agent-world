use std::collections::BTreeSet;

use agent_world::runtime::NodePointsRuntimeCollector;
use agent_world_node::NodeRole;

use super::reward_runtime_network::{
    reward_observation_trace_id, verify_reward_observation_trace, RewardObservationTrace,
};

pub(super) struct ObservationTraceApplyResult {
    pub report: Option<agent_world::runtime::EpochSettlementReport>,
    pub trace_id: String,
    pub observer_node_id: String,
    pub observer_role: NodeRole,
    pub payload_hash: String,
}

pub(super) fn observe_reward_observation_trace(
    collector: &mut NodePointsRuntimeCollector,
    trace: RewardObservationTrace,
    world_id: &str,
    source: &str,
    applied_trace_ids: &mut BTreeSet<String>,
    epoch_observer_nodes: &mut BTreeSet<String>,
) -> Option<ObservationTraceApplyResult> {
    if trace.version != 1 || trace.world_id != world_id {
        return None;
    }
    if let Err(err) = verify_reward_observation_trace(&trace) {
        eprintln!("reward runtime verify observation trace failed: {err}");
        return None;
    }
    let trace_id = match reward_observation_trace_id(&trace) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("reward runtime hash observation trace failed: {err}");
            return None;
        }
    };
    if applied_trace_ids.contains(trace_id.as_str()) {
        return None;
    }

    let observer_node_id = trace.observer_node_id.clone();
    let payload_hash = trace.payload_hash.clone();
    let observation = match trace.payload.into_observation() {
        Ok(observation) => observation,
        Err(err) => {
            eprintln!("reward runtime decode observation payload failed ({source}): {err}");
            return None;
        }
    };
    let observer_role = observation.role;
    let report = collector.observe(observation);
    applied_trace_ids.insert(trace_id.clone());
    epoch_observer_nodes.insert(observer_node_id.clone());
    Some(ObservationTraceApplyResult {
        report,
        trace_id,
        observer_node_id,
        observer_role,
        payload_hash,
    })
}
