//! Tests for the simulator module.

use super::*;
use crate::geometry::GeoPos;
use crate::models::DEFAULT_AGENT_HEIGHT_CM;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn pos(lat: f64, lon: f64) -> GeoPos {
    GeoPos {
        lat_deg: lat,
        lon_deg: lon,
    }
}

#[test]
fn resource_stock_add_remove() {
    let mut stock = ResourceStock::new();
    stock.add(ResourceKind::Electricity, 10).unwrap();
    stock.add(ResourceKind::Electricity, 5).unwrap();
    assert_eq!(stock.get(ResourceKind::Electricity), 15);

    stock.remove(ResourceKind::Electricity, 6).unwrap();
    assert_eq!(stock.get(ResourceKind::Electricity), 9);

    let err = stock.remove(ResourceKind::Electricity, 20).unwrap_err();
    assert!(matches!(err, StockError::Insufficient { .. }));
}

#[test]
fn agent_and_location_defaults() {
    let position = pos(0.0, 0.0);
    let location = Location::new("loc-1", "base", position);
    let agent = Agent::new("agent-1", "loc-1", position);

    assert_eq!(location.id, "loc-1");
    assert_eq!(agent.location_id, "loc-1");
    assert_eq!(agent.body.height_cm, DEFAULT_AGENT_HEIGHT_CM);
}

#[test]
fn world_model_starts_empty() {
    let model = WorldModel::default();
    assert!(model.agents.is_empty());
    assert!(model.locations.is_empty());
    assert!(model.assets.is_empty());
}

#[test]
fn kernel_registers_and_moves_agent() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step().unwrap();
    let starting_energy = 500;
    kernel
        .model()
        .agents
        .get("agent-1")
        .unwrap();
    // Need mutable access for this test - use a different approach
    let mut kernel2 = WorldKernel::new();
    kernel2.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel2.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel2.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel2.step_until_empty();

    // Give agent electricity via transfer from location
    kernel2.submit_action(Action::TransferResource {
        from: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        to: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        kind: ResourceKind::Electricity,
        amount: starting_energy,
    });
    // First need to add electricity to the location
    // For this test, use zero-cost config
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::AgentMoved {
            agent_id,
            from,
            to,
            distance_cm,
            electricity_cost,
        } => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(from, "loc-1");
            assert_eq!(to, "loc-2");
            assert!(distance_cm > 0);
            assert_eq!(electricity_cost, 0); // Zero-cost config
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel.model().agents.get("agent-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");
    assert_eq!(agent.pos, pos(1.0, 1.0));
}

#[test]
fn kernel_move_requires_energy() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(reason, RejectReason::InsufficientResource { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_rejects_move_to_same_location() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-1".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(reason, RejectReason::AgentAlreadyAtLocation { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_observe_visibility_range() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "near".to_string(),
        pos: pos(0.4, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-3".to_string(),
        name: "far".to_string(),
        pos: pos(1.5, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-2".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-3".to_string(),
        location_id: "loc-3".to_string(),
    });
    kernel.step_until_empty();

    let observation = kernel.observe("agent-1").unwrap();
    assert!(
        observation
            .visible_locations
            .iter()
            .any(|loc| loc.location_id == "loc-1")
    );
    assert!(
        observation
            .visible_locations
            .iter()
            .any(|loc| loc.location_id == "loc-2")
    );
    assert!(
        !observation
            .visible_locations
            .iter()
            .any(|loc| loc.location_id == "loc-3")
    );
    assert!(
        observation
            .visible_agents
            .iter()
            .any(|agent| agent.agent_id == "agent-2")
    );
    assert!(
        !observation
            .visible_agents
            .iter()
            .any(|agent| agent.agent_id == "agent-3")
    );
    assert_eq!(observation.visibility_range_cm, DEFAULT_VISIBILITY_RANGE_CM);
}

#[test]
fn kernel_config_overrides_defaults() {
    let config = WorldConfig {
        visibility_range_cm: CM_PER_KM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config.clone());
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.1, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let observation = kernel.observe("agent-1").unwrap();
    assert_eq!(observation.visibility_range_cm, config.visibility_range_cm);

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::AgentMoved { electricity_cost, .. } => {
            assert_eq!(electricity_cost, 0);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_snapshot_roundtrip() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let snapshot = kernel.snapshot();
    let journal = kernel.journal_snapshot();
    let restored = WorldKernel::from_snapshot(snapshot, journal).unwrap();

    assert_eq!(restored.time(), kernel.time());
    assert_eq!(restored.config(), kernel.config());
    assert_eq!(restored.model(), kernel.model());
    assert_eq!(restored.journal().len(), kernel.journal().len());
    assert_eq!(restored.pending_actions(), kernel.pending_actions());
}

#[test]
fn kernel_persist_and_restore() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("agent-world-sim-{unique}"));

    kernel.save_to_dir(&dir).unwrap();
    let restored = WorldKernel::load_from_dir(&dir).unwrap();

    assert_eq!(restored.model(), kernel.model());
    assert_eq!(restored.journal().len(), kernel.journal().len());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn restore_rejects_mismatched_journal_len() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.step_until_empty();

    let mut snapshot = kernel.snapshot();
    let mut journal = kernel.journal_snapshot();
    snapshot.journal_len = snapshot.journal_len + 1;
    journal.events.pop();

    let err = WorldKernel::from_snapshot(snapshot, journal).unwrap_err();
    assert!(matches!(err, PersistError::SnapshotMismatch { .. }));
}

#[test]
fn snapshot_version_validation_rejects_unknown() {
    let mut snapshot = WorldKernel::new().snapshot();
    snapshot.version = SNAPSHOT_VERSION + 1;
    let json = snapshot.to_json().unwrap();
    let err = WorldSnapshot::from_json(&json).unwrap_err();
    assert!(matches!(err, PersistError::UnsupportedVersion { .. }));
}

#[test]
fn journal_version_validation_rejects_unknown() {
    let mut journal = WorldKernel::new().journal_snapshot();
    journal.version = JOURNAL_VERSION + 1;
    let json = journal.to_json().unwrap();
    let err = WorldJournal::from_json(&json).unwrap_err();
    assert!(matches!(err, PersistError::UnsupportedVersion { .. }));
}

#[test]
fn kernel_replay_from_snapshot() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let snapshot = kernel.snapshot();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    kernel.step().unwrap();

    let journal = kernel.journal_snapshot();
    let replayed = WorldKernel::replay_from_snapshot(snapshot, journal).unwrap();

    assert_eq!(replayed.model(), kernel.model());
    assert_eq!(replayed.time(), kernel.time());
    assert_eq!(replayed.journal().len(), kernel.journal().len());
}

#[test]
fn kernel_transfer_requires_colocation() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-2".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::TransferResource {
        from: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        to: ResourceOwner::Agent {
            agent_id: "agent-2".to_string(),
        },
        kind: ResourceKind::Electricity,
        amount: 5,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(reason, RejectReason::AgentsNotCoLocated { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_closed_loop_example() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);
    let loc1_pos = pos(0.0, 0.0);
    let loc2_pos = pos(2.0, 2.0);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "plant".to_string(),
        pos: loc1_pos,
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "lab".to_string(),
        pos: loc2_pos,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-2".to_string(),
    });
    kernel.step_until_empty();

    // Move agent from loc-1 to loc-2
    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    kernel.step().unwrap();

    // Verify agent moved
    let agent = kernel.model().agents.get("agent-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");
}

// ========================================================================
// Agent Interface Tests
// ========================================================================

/// A simple test agent that moves toward a target location.
struct PatrolAgent {
    id: String,
    target_locations: Vec<String>,
    current_target_index: usize,
    action_results: Vec<bool>,
}

impl PatrolAgent {
    fn new(id: impl Into<String>, target_locations: Vec<String>) -> Self {
        Self {
            id: id.into(),
            target_locations,
            current_target_index: 0,
            action_results: Vec::new(),
        }
    }
}

impl AgentBehavior for PatrolAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        if self.target_locations.is_empty() {
            return AgentDecision::Wait;
        }

        let target_id = &self.target_locations[self.current_target_index];

        // Check if we're already at the target
        let current_location = observation
            .visible_locations
            .iter()
            .find(|loc| loc.distance_cm == 0);

        if let Some(current) = current_location {
            if &current.location_id == target_id {
                // Move to next target
                self.current_target_index =
                    (self.current_target_index + 1) % self.target_locations.len();
                let next_target = &self.target_locations[self.current_target_index];

                return AgentDecision::Act(Action::MoveAgent {
                    agent_id: self.id.clone(),
                    to: next_target.clone(),
                });
            }
        }

        // Move to current target
        AgentDecision::Act(Action::MoveAgent {
            agent_id: self.id.clone(),
            to: target_id.clone(),
        })
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        self.action_results.push(result.success);
    }
}

/// A simple agent that always waits.
struct WaitingAgent {
    id: String,
    wait_ticks: u64,
}

impl WaitingAgent {
    fn new(id: impl Into<String>, wait_ticks: u64) -> Self {
        Self {
            id: id.into(),
            wait_ticks,
        }
    }
}

impl AgentBehavior for WaitingAgent {
    fn agent_id(&self) -> &str {
        &self.id
    }

    fn decide(&mut self, _observation: &Observation) -> AgentDecision {
        if self.wait_ticks > 0 {
            AgentDecision::WaitTicks(self.wait_ticks)
        } else {
            AgentDecision::Wait
        }
    }
}

#[test]
fn agent_decision_helpers() {
    let wait = AgentDecision::Wait;
    assert!(!wait.is_act());
    assert!(wait.action().is_none());

    let wait_ticks = AgentDecision::WaitTicks(5);
    assert!(!wait_ticks.is_act());
    assert!(wait_ticks.action().is_none());

    let action = Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    };
    let act = AgentDecision::Act(action.clone());
    assert!(act.is_act());
    assert_eq!(act.action(), Some(&action));
}

#[test]
fn action_result_helpers() {
    let action = Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    };

    // Success result
    let success_event = WorldEvent {
        id: 1,
        time: 1,
        kind: WorldEventKind::AgentMoved {
            agent_id: "agent-1".to_string(),
            from: "loc-1".to_string(),
            to: "loc-2".to_string(),
            distance_cm: 1000,
            electricity_cost: 1,
        },
    };
    let success_result = ActionResult {
        action: action.clone(),
        action_id: 1,
        success: true,
        event: success_event,
    };
    assert!(!success_result.is_rejected());
    assert!(success_result.reject_reason().is_none());

    // Rejected result
    let reject_event = WorldEvent {
        id: 2,
        time: 2,
        kind: WorldEventKind::ActionRejected {
            reason: RejectReason::AgentNotFound {
                agent_id: "agent-1".to_string(),
            },
        },
    };
    let reject_result = ActionResult {
        action,
        action_id: 2,
        success: false,
        event: reject_event,
    };
    assert!(reject_result.is_rejected());
    assert!(matches!(
        reject_result.reject_reason(),
        Some(RejectReason::AgentNotFound { .. })
    ));
}

#[test]
fn agent_runner_register_and_tick() {
    // Zero-cost movement config for simpler testing
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    // Setup world
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "patrol-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    // Create agent runner
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    let patrol_agent = PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    );
    runner.register(patrol_agent);

    assert_eq!(runner.agent_count(), 1);
    assert_eq!(runner.agent_ids(), vec!["patrol-1".to_string()]);

    // Run one tick - agent should move from loc-1 to loc-2
    let result = runner.tick(&mut kernel);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.agent_id, "patrol-1");
    assert!(result.has_action());
    assert!(result.is_success());

    // Verify agent moved
    let agent = kernel.model().agents.get("patrol-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");

    // Check agent stats
    let registered = runner.get("patrol-1").unwrap();
    assert_eq!(registered.action_count, 1);
    assert_eq!(registered.decision_count, 1);
    assert_eq!(registered.behavior.action_results.len(), 1);
    assert!(registered.behavior.action_results[0]);
}

#[test]
fn agent_runner_round_robin() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    // Setup world with two agents
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-a".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-b".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.register(PatrolAgent::new(
        "agent-a",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));
    runner.register(PatrolAgent::new(
        "agent-b",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Run ticks - should alternate between agents
    let result1 = runner.tick(&mut kernel).unwrap();
    let result2 = runner.tick(&mut kernel).unwrap();
    let result3 = runner.tick(&mut kernel).unwrap();

    // Round-robin: agent-a, agent-b, agent-a
    assert_eq!(result1.agent_id, "agent-a");
    assert_eq!(result2.agent_id, "agent-b");
    assert_eq!(result3.agent_id, "agent-a");
}

#[test]
fn agent_runner_wait_ticks() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "waiter".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.register(WaitingAgent::new("waiter", 3));

    // First tick - agent decides to wait 3 ticks
    let result = runner.tick(&mut kernel).unwrap();
    assert_eq!(result.decision, AgentDecision::WaitTicks(3));
    assert!(!result.has_action());

    // Agent should not be ready for the next 3 ticks
    let registered = runner.get("waiter").unwrap();
    assert!(registered.wait_until.is_some());
}

#[test]
fn agent_runner_run_multiple_ticks() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "patrol-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Run 4 ticks
    let results = runner.run(&mut kernel, 4);
    assert_eq!(results.len(), 4);
    assert_eq!(runner.total_ticks(), 4);

    // Each result should have an action
    for result in &results {
        assert!(result.has_action());
    }

    // Agent should have patrolled back and forth
    let registered = runner.get("patrol-1").unwrap();
    assert_eq!(registered.action_count, 4);
}

#[test]
fn agent_runner_unregister() {
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.register(PatrolAgent::new("agent-1", vec![]));
    runner.register(PatrolAgent::new("agent-2", vec![]));

    assert_eq!(runner.agent_count(), 2);

    let removed = runner.unregister("agent-1");
    assert!(removed.is_some());
    assert_eq!(runner.agent_count(), 1);
    assert!(runner.get("agent-1").is_none());
    assert!(runner.get("agent-2").is_some());
}

#[test]
fn registered_agent_is_ready() {
    let agent = RegisteredAgent::new(PatrolAgent::new("test", vec![]));
    assert!(agent.is_ready(0));
    assert!(agent.is_ready(100));

    let mut waiting_agent = RegisteredAgent::new(PatrolAgent::new("test2", vec![]));
    waiting_agent.wait_until = Some(10);
    assert!(!waiting_agent.is_ready(5));
    assert!(waiting_agent.is_ready(10));
    assert!(waiting_agent.is_ready(15));
}

// ========================================================================
// Quota and Rate Limiting Tests
// ========================================================================

#[test]
fn agent_quota_max_actions() {
    let quota = AgentQuota::max_actions(5);
    assert!(!quota.is_exhausted(0, 0));
    assert!(!quota.is_exhausted(4, 0));
    assert!(quota.is_exhausted(5, 0));
    assert!(quota.is_exhausted(10, 0));

    assert_eq!(quota.remaining_actions(3), Some(2));
    assert_eq!(quota.remaining_actions(5), Some(0));
    assert_eq!(quota.remaining_decisions(10), None);
}

#[test]
fn agent_quota_max_decisions() {
    let quota = AgentQuota::max_decisions(3);
    assert!(!quota.is_exhausted(0, 0));
    assert!(!quota.is_exhausted(100, 2));
    assert!(quota.is_exhausted(100, 3));
    assert!(quota.is_exhausted(0, 5));

    assert_eq!(quota.remaining_decisions(1), Some(2));
    assert_eq!(quota.remaining_actions(100), None);
}

#[test]
fn agent_quota_both_limits() {
    let quota = AgentQuota::new(Some(5), Some(10));
    assert!(!quota.is_exhausted(4, 9));
    assert!(quota.is_exhausted(5, 9)); // actions exhausted
    assert!(quota.is_exhausted(4, 10)); // decisions exhausted
    assert!(quota.is_exhausted(5, 10)); // both exhausted
}

#[test]
fn rate_limit_policy_actions_per_tick() {
    let policy = RateLimitPolicy::actions_per_tick(2);
    assert_eq!(policy.max_actions_per_window, 2);
    assert_eq!(policy.window_size_ticks, 1);
}

#[test]
fn rate_limit_state_basic() {
    let policy = RateLimitPolicy::new(2, 5); // 2 actions per 5 ticks
    let mut state = RateLimitState::default();

    // Initially not limited
    assert!(!state.is_limited(0, &policy));

    // Record first action
    state.record_action(0, &policy);
    assert!(!state.is_limited(0, &policy));

    // Record second action - should hit the limit
    state.record_action(0, &policy);
    assert!(state.is_limited(0, &policy));
    assert!(state.is_limited(4, &policy)); // Still in same window

    // New window should reset
    assert!(!state.is_limited(5, &policy));

    // Recording action in new window resets count
    state.record_action(5, &policy);
    assert!(!state.is_limited(5, &policy));
    assert_eq!(state.window_start, 5);
    assert_eq!(state.actions_in_window, 1);
}

#[test]
fn rate_limit_state_reset() {
    let policy = RateLimitPolicy::new(1, 10);
    let mut state = RateLimitState::default();

    state.record_action(0, &policy);
    assert!(state.is_limited(0, &policy));

    state.reset();
    assert!(!state.is_limited(0, &policy));
    assert_eq!(state.window_start, 0);
    assert_eq!(state.actions_in_window, 0);
}

#[test]
fn agent_runner_with_quota() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "patrol-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    // Create runner with default quota of 3 actions
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::with_quota(AgentQuota::max_actions(3));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Run 5 ticks - should only get 3 results due to quota
    let mut action_count = 0;
    for _ in 0..5 {
        if let Some(result) = runner.tick(&mut kernel) {
            if result.has_action() {
                action_count += 1;
            }
        }
    }
    assert_eq!(action_count, 3);
    assert!(runner.is_quota_exhausted("patrol-1"));
}

#[test]
fn agent_runner_with_rate_limit() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "patrol-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    // Create runner with rate limit of 1 action per 10 ticks
    let mut runner: AgentRunner<PatrolAgent> =
        AgentRunner::with_rate_limit(RateLimitPolicy::new(1, 10));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // First tick should succeed
    let result = runner.tick(&mut kernel);
    assert!(result.is_some());
    assert!(result.as_ref().unwrap().has_action());

    // Second tick should be rate-limited (no agent ready)
    let result = runner.tick(&mut kernel);
    assert!(result.is_none());

    // Agent should be rate-limited
    let now = kernel.time();
    assert!(runner.is_rate_limited("patrol-1", now));
}

#[test]
fn agent_runner_per_agent_quota() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-a".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-b".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    // agent-a has quota of 2 actions
    runner.register_with_quota(
        PatrolAgent::new("agent-a", vec!["loc-1".to_string(), "loc-2".to_string()]),
        AgentQuota::max_actions(2),
    );
    // agent-b has no quota (unlimited)
    runner.register(PatrolAgent::new(
        "agent-b",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Run 10 ticks
    let results = runner.run(&mut kernel, 10);

    // agent-a should have only 2 actions
    let agent_a = runner.get("agent-a").unwrap();
    assert_eq!(agent_a.action_count, 2);
    assert!(runner.is_quota_exhausted("agent-a"));

    // agent-b should have more actions (limited only by round-robin)
    let agent_b = runner.get("agent-b").unwrap();
    assert!(agent_b.action_count > 2);
    assert!(!runner.is_quota_exhausted("agent-b"));

    // Should have gotten results for all ticks
    assert!(!results.is_empty());
}

#[test]
fn agent_runner_reset_rate_limits() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "patrol-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> =
        AgentRunner::with_rate_limit(RateLimitPolicy::new(1, 100));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // First action
    runner.tick(&mut kernel);
    let now = kernel.time();
    assert!(runner.is_rate_limited("patrol-1", now));

    // Reset rate limit
    runner.reset_rate_limit("patrol-1");
    assert!(!runner.is_rate_limited("patrol-1", now));

    // Can take another action
    let result = runner.tick(&mut kernel);
    assert!(result.is_some());
    assert!(result.unwrap().has_action());
}

// ========================================================================
// Observability and Metrics Tests
// ========================================================================

#[test]
fn runner_metrics_basic() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.register(PatrolAgent::new(
        "agent-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));
    runner.register(PatrolAgent::new(
        "agent-2",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Initial metrics
    let metrics = runner.metrics();
    assert_eq!(metrics.total_ticks, 0);
    assert_eq!(metrics.total_agents, 2);
    assert_eq!(metrics.agents_active, 2);
    assert_eq!(metrics.agents_quota_exhausted, 0);
    assert_eq!(metrics.total_actions, 0);
    assert_eq!(metrics.total_decisions, 0);

    // Run some ticks
    runner.run(&mut kernel, 4);

    // Check metrics after running
    let metrics = runner.metrics();
    assert_eq!(metrics.total_ticks, 4);
    assert_eq!(metrics.total_agents, 2);
    assert!(metrics.total_actions > 0);
    assert!(metrics.total_decisions > 0);
    assert!(metrics.actions_per_tick > 0.0);
    assert!(metrics.decisions_per_tick > 0.0);
}

#[test]
fn runner_metrics_with_quota() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    // Create runner with quota of 2 actions
    let mut runner: AgentRunner<PatrolAgent> =
        AgentRunner::with_quota(AgentQuota::max_actions(2));
    runner.register(PatrolAgent::new(
        "agent-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Run until quota exhausted
    runner.run(&mut kernel, 5);

    let metrics = runner.metrics();
    assert_eq!(metrics.agents_quota_exhausted, 1);
    assert_eq!(metrics.agents_active, 0); // All agents quota-exhausted
    assert_eq!(metrics.total_actions, 2); // Limited by quota
}

#[test]
fn runner_agent_stats() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
    };
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.01, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-a".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-b".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.register_with_quota(
        PatrolAgent::new("agent-a", vec!["loc-1".to_string(), "loc-2".to_string()]),
        AgentQuota::max_actions(1),
    );
    runner.register(PatrolAgent::new(
        "agent-b",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    // Run a few ticks
    runner.run(&mut kernel, 4);

    // Check per-agent stats
    let stats = runner.agent_stats();
    assert_eq!(stats.len(), 2);

    let agent_a_stats = stats.iter().find(|s| s.agent_id == "agent-a").unwrap();
    assert_eq!(agent_a_stats.action_count, 1);
    assert!(agent_a_stats.is_quota_exhausted);

    let agent_b_stats = stats.iter().find(|s| s.agent_id == "agent-b").unwrap();
    assert!(agent_b_stats.action_count >= 1);
    assert!(!agent_b_stats.is_quota_exhausted);
}

#[test]
fn runner_log_entry_serialization() {
    let entry = RunnerLogEntry {
        tick: 10,
        time: 100,
        kind: RunnerLogKind::AgentRegistered {
            agent_id: "test-agent".to_string(),
        },
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("AgentRegistered"));
    assert!(json.contains("test-agent"));

    let parsed: RunnerLogEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.tick, 10);
    assert_eq!(parsed.time, 100);
}

#[test]
fn runner_metrics_default() {
    let metrics = RunnerMetrics::default();
    assert_eq!(metrics.total_ticks, 0);
    assert_eq!(metrics.total_agents, 0);
    assert_eq!(metrics.agents_active, 0);
    assert_eq!(metrics.total_actions, 0);
    assert_eq!(metrics.actions_per_tick, 0.0);
    assert_eq!(metrics.success_rate, 0.0);
}

// ========================================================================
// Agent Memory Tests
// ========================================================================

#[test]
fn short_term_memory_basic() {
    let mut mem = ShortTermMemory::new(5);
    assert!(mem.is_empty());
    assert_eq!(mem.capacity(), 5);

    // Add some entries
    mem.add(MemoryEntry::observation(1, "Saw agent-b"));
    mem.add(MemoryEntry::decision(2, AgentDecision::Wait));
    mem.add(MemoryEntry::note(3, "Feeling good"));

    assert_eq!(mem.len(), 3);
    assert_eq!(mem.total_added(), 3);

    // Recent entries
    let recent: Vec<_> = mem.recent(2).collect();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].time, 3); // Most recent first
    assert_eq!(recent[1].time, 2);
}

#[test]
fn short_term_memory_capacity_eviction() {
    let mut mem = ShortTermMemory::new(3);

    mem.add(MemoryEntry::observation(1, "First"));
    mem.add(MemoryEntry::observation(2, "Second"));
    mem.add(MemoryEntry::observation(3, "Third"));
    assert_eq!(mem.len(), 3);

    // Adding a 4th should evict the first
    mem.add(MemoryEntry::observation(4, "Fourth"));
    assert_eq!(mem.len(), 3);
    assert_eq!(mem.total_added(), 4);

    let all: Vec<_> = mem.all().collect();
    assert_eq!(all[0].time, 2); // First was evicted
    assert_eq!(all[2].time, 4);
}

#[test]
fn short_term_memory_since_filter() {
    let mut mem = ShortTermMemory::new(10);

    mem.add(MemoryEntry::observation(5, "Early"));
    mem.add(MemoryEntry::observation(10, "Middle"));
    mem.add(MemoryEntry::observation(15, "Late"));

    let since_10: Vec<_> = mem.since(10).collect();
    assert_eq!(since_10.len(), 2);
    assert!(since_10.iter().all(|e| e.time >= 10));
}

#[test]
fn short_term_memory_importance_filter() {
    let mut mem = ShortTermMemory::new(10);

    mem.add(MemoryEntry::observation(1, "Low importance").with_importance(0.3));
    mem.add(MemoryEntry::observation(2, "High importance").with_importance(0.9));
    mem.add(MemoryEntry::observation(3, "Medium importance").with_importance(0.5));

    let important: Vec<_> = mem.important(0.7).collect();
    assert_eq!(important.len(), 1);
    assert_eq!(important[0].time, 2);
}

#[test]
fn short_term_memory_summarize() {
    let mut mem = ShortTermMemory::new(10);

    mem.add(MemoryEntry::observation(1, "Saw base"));
    mem.add(MemoryEntry::note(2, "Planning to move"));

    let summary = mem.summarize(5);
    assert!(summary.contains("[T1]"));
    assert!(summary.contains("Saw base"));
    assert!(summary.contains("[T2]"));
    assert!(summary.contains("Planning to move"));
}

#[test]
fn long_term_memory_basic() {
    let mut mem = LongTermMemory::new();
    assert!(mem.is_empty());

    let id1 = mem.store("Important discovery", 10);
    let _id2 = mem.store("Another fact", 20);

    assert_eq!(mem.len(), 2);

    // Retrieve and check
    let entry = mem.get(&id1, 25).unwrap();
    assert_eq!(entry.content, "Important discovery");
    assert_eq!(entry.created_at, 10);
    assert_eq!(entry.last_accessed, 25);
    assert_eq!(entry.access_count, 1);
}

#[test]
fn long_term_memory_search_by_tag() {
    let mut mem = LongTermMemory::new();

    mem.store_with_tags("Location info", 1, vec!["location".to_string()]);
    mem.store_with_tags("Agent info", 2, vec!["agent".to_string()]);
    mem.store_with_tags("Both", 3, vec!["location".to_string(), "agent".to_string()]);

    let location_results = mem.search_by_tag("location");
    assert_eq!(location_results.len(), 2);

    let agent_results = mem.search_by_tag("agent");
    assert_eq!(agent_results.len(), 2);
}

#[test]
fn long_term_memory_search_by_content() {
    let mut mem = LongTermMemory::new();

    mem.store("The base is at coordinates 0,0", 1);
    mem.store("Agent-1 is friendly", 2);
    mem.store("The outpost has resources", 3);

    let results = mem.search_by_content("base");
    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("base"));

    let results_case = mem.search_by_content("AGENT");
    assert_eq!(results_case.len(), 1); // Case-insensitive
}

#[test]
fn long_term_memory_capacity_eviction() {
    let mut mem = LongTermMemory::with_capacity(3);

    let id1 = mem.store("Low importance", 1);
    if let Some(e) = mem.entries.get_mut(&id1) {
        e.importance = 0.1;
    }

    let id2 = mem.store("High importance", 2);
    if let Some(e) = mem.entries.get_mut(&id2) {
        e.importance = 0.9;
    }

    let id3 = mem.store("Medium importance", 3);
    if let Some(e) = mem.entries.get_mut(&id3) {
        e.importance = 0.5;
    }

    assert_eq!(mem.len(), 3);

    // Adding a 4th should evict the lowest importance
    let id4 = mem.store("New entry", 4);
    if let Some(e) = mem.entries.get_mut(&id4) {
        e.importance = 0.6;
    }

    assert_eq!(mem.len(), 3);
    assert!(mem.entries.get(&id1).is_none()); // Low importance was evicted
    assert!(mem.entries.get(&id2).is_some()); // High importance kept
}

#[test]
fn long_term_memory_top_by_importance() {
    let mut mem = LongTermMemory::new();

    let id1 = mem.store("Low", 1);
    mem.entries.get_mut(&id1).unwrap().importance = 0.2;

    let id2 = mem.store("High", 2);
    mem.entries.get_mut(&id2).unwrap().importance = 0.9;

    let id3 = mem.store("Medium", 3);
    mem.entries.get_mut(&id3).unwrap().importance = 0.5;

    let top = mem.top_by_importance(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].importance, 0.9);
    assert_eq!(top[1].importance, 0.5);
}

#[test]
fn agent_memory_combined() {
    let mut memory = AgentMemory::new();

    // Record some short-term memories
    memory.record_observation(1, "Saw the base");
    memory.record_decision(2, AgentDecision::Act(Action::MoveAgent {
        agent_id: "test".to_string(),
        to: "base".to_string(),
    }));
    memory.record_action_result(
        3,
        Action::MoveAgent {
            agent_id: "test".to_string(),
            to: "base".to_string(),
        },
        true,
    );
    memory.record_event(4, "Arrived at destination");

    assert_eq!(memory.short_term.len(), 4);
    assert!(memory.long_term.is_empty());

    // Get context summary
    let context = memory.context_summary(3);
    assert!(context.contains("Arrived at destination"));
    assert!(context.contains("Action"));
}

#[test]
fn agent_memory_consolidation() {
    let mut memory = AgentMemory::new();

    // Add memories with varying importance
    memory.short_term.add(MemoryEntry::observation(1, "Low").with_importance(0.3));
    memory.short_term.add(MemoryEntry::observation(2, "High").with_importance(0.9));
    memory.short_term.add(MemoryEntry::observation(3, "Medium").with_importance(0.5));

    // Consolidate high-importance memories
    memory.consolidate(10, 0.8);

    // High importance should be in long-term memory
    assert_eq!(memory.long_term.len(), 1);
    let results = memory.long_term.search_by_content("High");
    assert_eq!(results.len(), 1);
}

#[test]
fn memory_entry_serialization() {
    let entry = MemoryEntry::observation(10, "Test observation").with_importance(0.7);

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("Observation"));
    assert!(json.contains("Test observation"));

    let parsed: MemoryEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.time, 10);
    assert_eq!(parsed.importance, 0.7);
}
