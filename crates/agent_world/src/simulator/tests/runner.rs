use super::*;

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

        let current_location = observation
            .visible_locations
            .iter()
            .find(|loc| loc.distance_cm == 0);

        if let Some(current) = current_location {
            if &current.location_id == target_id {
                self.current_target_index =
                    (self.current_target_index + 1) % self.target_locations.len();
                let next_target = &self.target_locations[self.current_target_index];

                return AgentDecision::Act(Action::MoveAgent {
                    agent_id: self.id.clone(),
                    to: next_target.clone(),
                });
            }
        }

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

fn setup_kernel_with_patrol_agent(agent_id: &str) -> WorldKernel {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
        ..Default::default()
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
        agent_id: agent_id.to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel
}

fn setup_kernel_with_wait_agent(agent_id: &str) -> WorldKernel {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: agent_id.to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();
    kernel
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
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    let patrol_agent = PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    );
    runner.register(patrol_agent);

    assert_eq!(runner.agent_count(), 1);
    assert_eq!(runner.agent_ids(), vec!["patrol-1".to_string()]);

    let result = runner.tick(&mut kernel);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.agent_id, "patrol-1");
    assert!(result.has_action());
    assert!(result.is_success());

    let agent = kernel.model().agents.get("patrol-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");

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
        ..Default::default()
    };
    let mut kernel = WorldKernel::with_config(config);
    for idx in 0..3 {
        kernel.submit_action(Action::RegisterLocation {
            location_id: format!("loc-{idx}"),
            name: format!("loc-{idx}"),
            pos: pos(idx as f64 * 0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: format!("agent-{idx}"),
            location_id: format!("loc-{idx}"),
        });
    }
    kernel.step_until_empty();

    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    for idx in 0..3 {
        let agent = PatrolAgent::new(
            format!("agent-{idx}"),
            vec![format!("loc-{idx}")],
        );
        runner.register(agent);
    }

    let mut seen = Vec::new();
    for _ in 0..3 {
        let tick = runner.tick(&mut kernel).unwrap();
        seen.push(tick.agent_id);
    }

    assert_eq!(seen, vec!["agent-0", "agent-1", "agent-2"]);
}

#[test]
fn agent_runner_wait_ticks_sets_wait_until() {
    let mut kernel = setup_kernel_with_wait_agent("agent-1");
    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.register(WaitingAgent::new("agent-1", 2));

    let now = kernel.time();
    let tick = runner.tick(&mut kernel).unwrap();
    assert!(matches!(tick.decision, AgentDecision::WaitTicks(2)));
    assert!(tick.action_result.is_none());

    let registered = runner.get("agent-1").unwrap();
    assert_eq!(registered.wait_until, Some(now.saturating_add(2)));
}

#[test]
fn agent_runner_run_multiple_ticks() {
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    let patrol_agent = PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    );
    runner.register(patrol_agent);

    for _ in 0..3 {
        runner.tick(&mut kernel).unwrap();
    }

    let agent = kernel.model().agents.get("patrol-1").unwrap();
    assert!(agent.location_id == "loc-1" || agent.location_id == "loc-2");
}

#[test]
fn agent_runner_unregister() {
    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.register(WaitingAgent::new("agent-1", 0));
    assert_eq!(runner.agent_count(), 1);

    runner.unregister("agent-1");
    assert_eq!(runner.agent_count(), 0);
}

#[test]
fn registered_agent_is_ready() {
    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.register(WaitingAgent::new("agent-1", 0));
    let registered = runner.get("agent-1").unwrap();
    assert!(registered.is_ready(1));
}

#[test]
fn agent_quota_max_actions() {
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.set_default_quota(Some(AgentQuota::max_actions(1)));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    let tick1 = runner.tick(&mut kernel).unwrap();
    assert!(tick1.has_action());
    assert!(runner.tick(&mut kernel).is_none());
}

#[test]
fn agent_quota_max_decisions() {
    let quota = AgentQuota::max_decisions(1);
    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.set_default_quota(Some(quota));
    runner.register(WaitingAgent::new("agent-1", 0));
    let mut kernel = setup_kernel_with_wait_agent("agent-1");

    let tick1 = runner.tick(&mut kernel).unwrap();
    assert!(matches!(tick1.decision, AgentDecision::Wait));
    assert!(runner.tick(&mut kernel).is_none());
}

#[test]
fn agent_quota_both_limits() {
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.set_default_quota(Some(AgentQuota::new(Some(1), Some(1))));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    let tick1 = runner.tick(&mut kernel).unwrap();
    assert!(tick1.has_action());
    assert!(runner.tick(&mut kernel).is_none());
}

#[test]
fn rate_limit_policy_actions_per_tick() {
    let policy = RateLimitPolicy::new(1, 2);
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.set_rate_limit(Some(policy));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");

    let tick1 = runner.tick(&mut kernel).unwrap();
    assert!(tick1.has_action());
    assert!(runner.is_rate_limited("patrol-1", kernel.time()));

    runner.reset_rate_limit("patrol-1");
    assert!(!runner.is_rate_limited("patrol-1", kernel.time()));
}

#[test]
fn rate_limit_state_basic() {
    let policy = RateLimitPolicy::new(1, 2);
    let mut state = RateLimitState::default();

    assert!(!state.is_limited(0, &policy));
    state.record_action(0, &policy);
    assert!(state.is_limited(0, &policy));
    assert!(state.is_limited(1, &policy));
    assert!(!state.is_limited(2, &policy));
}

#[test]
fn rate_limit_state_reset() {
    let policy = RateLimitPolicy::new(1, 2);
    let mut state = RateLimitState::default();
    state.record_action(0, &policy);
    assert!(state.is_limited(1, &policy));

    state.reset();
    assert!(!state.is_limited(1, &policy));
}

#[test]
fn agent_runner_per_agent_quota() {
    let mut kernel = setup_kernel_with_wait_agent("agent-1");
    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.register_with_quota(WaitingAgent::new("agent-1", 0), AgentQuota::max_decisions(1));

    let tick1 = runner.tick(&mut kernel).unwrap();
    assert!(matches!(tick1.decision, AgentDecision::Wait));
    assert!(runner.tick(&mut kernel).is_none());
}

#[test]
fn runner_metrics_basic() {
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    runner.tick(&mut kernel);
    runner.tick(&mut kernel);

    let metrics = runner.metrics();
    assert_eq!(metrics.total_ticks, 2);
    assert_eq!(metrics.total_agents, 1);
    assert_eq!(metrics.total_actions, 2);
    assert_eq!(metrics.total_decisions, 2);
}

#[test]
fn runner_metrics_with_quota() {
    let mut kernel = setup_kernel_with_patrol_agent("patrol-1");
    let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
    runner.set_default_quota(Some(AgentQuota::max_actions(1)));
    runner.register(PatrolAgent::new(
        "patrol-1",
        vec!["loc-1".to_string(), "loc-2".to_string()],
    ));

    runner.tick(&mut kernel);
    runner.tick(&mut kernel);

    let metrics = runner.metrics();
    assert_eq!(metrics.total_ticks, 2);
    assert_eq!(metrics.total_actions, 1);
    assert_eq!(metrics.agents_quota_exhausted, 1);
}

#[test]
fn runner_agent_stats() {
    let mut kernel = setup_kernel_with_wait_agent("agent-1");
    let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
    runner.register(WaitingAgent::new("agent-1", 0));

    runner.tick(&mut kernel);

    let stats = runner.agent_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].agent_id, "agent-1");
    assert_eq!(stats[0].action_count, 0);
    assert_eq!(stats[0].decision_count, 1);
    assert!(!stats[0].is_quota_exhausted);
    assert_eq!(stats[0].wait_until, None);
}

#[test]
fn runner_log_entry_serialization() {
    let entry = RunnerLogEntry {
        tick: 1,
        time: 1,
        kind: RunnerLogKind::ActionExecuted {
            agent_id: "agent-1".to_string(),
            action: Action::MoveAgent {
                agent_id: "agent-1".to_string(),
                to: "loc-1".to_string(),
            },
            success: true,
        },
    };
    let serialized = serde_json::to_string(&entry).unwrap();
    assert!(serialized.contains("agent-1"));
}

#[test]
fn runner_metrics_default() {
    let metrics = RunnerMetrics::default();
    assert_eq!(metrics.total_ticks, 0);
    assert_eq!(metrics.total_agents, 0);
    assert_eq!(metrics.total_actions, 0);
    assert_eq!(metrics.total_decisions, 0);
}
