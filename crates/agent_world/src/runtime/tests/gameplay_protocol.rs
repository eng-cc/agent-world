use super::super::*;
use super::pos;
use crate::simulator::ResourceKind;
use agent_world_wasm_abi::{
    ModuleCallFailure, ModuleCallRequest, ModuleEmit, ModuleOutput, ModuleSandbox,
    ModuleTickLifecycleDirective,
};

struct GameplayDirectiveSandbox {
    governance_directives: Vec<serde_json::Value>,
}

impl GameplayDirectiveSandbox {
    fn empty() -> Self {
        Self {
            governance_directives: Vec::new(),
        }
    }

    fn with_governance_directive(payload: serde_json::Value) -> Self {
        Self {
            governance_directives: vec![payload],
        }
    }
}

impl ModuleSandbox for GameplayDirectiveSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let mut emits = Vec::new();
        if request.module_id == M5_GAMEPLAY_GOVERNANCE_MODULE_ID
            && (request.trace_id.starts_with("tick-")
                || request.trace_id.starts_with("infra-tick-"))
        {
            if let Some(payload) = self.governance_directives.pop() {
                emits.push(ModuleEmit {
                    kind: "gameplay.lifecycle.directives".to_string(),
                    payload,
                });
            }
        }
        Ok(ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits,
            tick_lifecycle: Some(ModuleTickLifecycleDirective::WakeAfterTicks { ticks: 1 }),
            output_bytes: 512,
        })
    }
}

fn register_agents(world: &mut World, agent_ids: &[&str]) {
    for (index, agent_id) in agent_ids.iter().enumerate() {
        world.submit_action(Action::RegisterAgent {
            agent_id: (*agent_id).to_string(),
            pos: pos(index as f64, 0.0),
        });
    }
    world.step().expect("register agents");
}

fn last_domain_event(world: &World) -> &DomainEvent {
    let event = world.journal().events.last().expect("domain event");
    let WorldEventBody::Domain(domain_event) = &event.body else {
        panic!("expected domain event");
    };
    domain_event
}

fn open_governance_proposal(
    world: &mut World,
    proposal_key: &str,
    window_ticks: u64,
    quorum_weight: u64,
    pass_threshold_bps: u16,
) {
    world.submit_action(Action::OpenGovernanceProposal {
        proposer_agent_id: "a".to_string(),
        proposal_key: proposal_key.to_string(),
        title: format!("title.{proposal_key}"),
        description: "runtime proposal".to_string(),
        options: vec!["approve".to_string(), "reject".to_string()],
        voting_window_ticks: window_ticks,
        quorum_weight,
        pass_threshold_bps,
    });
    world.step().expect("open governance proposal");
}

fn advance_until_auto_crisis(world: &mut World) -> String {
    for _ in 0..64 {
        world.step().expect("advance for crisis cycle");
        if let Some((crisis_id, _)) = world
            .state()
            .crises
            .iter()
            .find(|(_, crisis)| crisis.status == CrisisStatus::Active)
        {
            return crisis_id.clone();
        }
    }
    panic!("expected an auto crisis to spawn");
}

#[test]
fn gameplay_protocol_actions_drive_persisted_state() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c", "d"]);

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "a".to_string(),
        alliance_id: "alliance.red".to_string(),
        members: vec!["b".to_string()],
        charter: "mutual defense".to_string(),
    });
    world.step().expect("form red alliance");

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "c".to_string(),
        alliance_id: "alliance.blue".to_string(),
        members: vec!["d".to_string()],
        charter: "logistics pact".to_string(),
    });
    world.step().expect("form blue alliance");

    world.submit_action(Action::DeclareWar {
        initiator_agent_id: "a".to_string(),
        war_id: "war.001".to_string(),
        aggressor_alliance_id: "alliance.red".to_string(),
        defender_alliance_id: "alliance.blue".to_string(),
        objective: "control asteroid belt".to_string(),
        intensity: 2,
    });
    world.step().expect("declare war");

    open_governance_proposal(&mut world, "proposal.energy_tax", 4, 2, 5_000);

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.energy_tax".to_string(),
        option: "approve".to_string(),
        weight: 3,
    });
    world.step().expect("cast governance vote");

    let crisis_id = advance_until_auto_crisis(&mut world);
    world.submit_action(Action::ResolveCrisis {
        resolver_agent_id: "c".to_string(),
        crisis_id: crisis_id.clone(),
        strategy: "redistribute shield grid".to_string(),
        success: true,
    });
    world.step().expect("resolve crisis");

    world.submit_action(Action::GrantMetaProgress {
        operator_agent_id: "a".to_string(),
        target_agent_id: "b".to_string(),
        track: "campaign".to_string(),
        points: 15,
        achievement_id: Some("first_alliance_win".to_string()),
    });
    world.step().expect("grant meta progress");

    let red = world
        .state()
        .alliances
        .get("alliance.red")
        .expect("red alliance");
    assert_eq!(red.members, vec!["a".to_string(), "b".to_string()]);

    let war = world.state().wars.get("war.001").expect("war record");
    assert_eq!(war.aggressor_alliance_id, "alliance.red");
    assert_eq!(war.defender_alliance_id, "alliance.blue");
    assert!(war.active);

    let governance = world
        .state()
        .governance_votes
        .get("proposal.energy_tax")
        .expect("governance vote state");
    assert_eq!(governance.total_weight, 3);
    assert_eq!(governance.tallies.get("approve"), Some(&3_u64));

    let proposal = world
        .state()
        .governance_proposals
        .get("proposal.energy_tax")
        .expect("governance proposal state");
    assert_eq!(proposal.status, GovernanceProposalStatus::Passed);

    let crisis = world.state().crises.get(&crisis_id).expect("crisis state");
    assert_eq!(crisis.status, CrisisStatus::Resolved);
    assert_eq!(crisis.success, Some(true));
    assert_eq!(crisis.impact, 20);

    let progress = world.state().meta_progress.get("b").expect("meta progress");
    assert_eq!(progress.total_points, 15);
    assert_eq!(progress.track_points.get("campaign"), Some(&15));
    assert_eq!(
        progress.achievements,
        vec!["first_alliance_win".to_string()]
    );

    assert!(matches!(
        last_domain_event(&world),
        DomainEvent::MetaProgressGranted { .. }
    ));
}

#[test]
fn declare_war_rejects_initiator_outside_aggressor_alliance() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c", "d"]);

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "a".to_string(),
        alliance_id: "alliance.red".to_string(),
        members: vec!["b".to_string()],
        charter: "charter.red".to_string(),
    });
    world.step().expect("form red alliance");

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "c".to_string(),
        alliance_id: "alliance.blue".to_string(),
        members: vec!["d".to_string()],
        charter: "charter.blue".to_string(),
    });
    world.step().expect("form blue alliance");

    world.submit_action(Action::DeclareWar {
        initiator_agent_id: "c".to_string(),
        war_id: "war.invalid".to_string(),
        aggressor_alliance_id: "alliance.red".to_string(),
        defender_alliance_id: "alliance.blue".to_string(),
        objective: "invalid".to_string(),
        intensity: 1,
    });
    world.step().expect("reject invalid war declare");

    match last_domain_event(&world) {
        DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        } => {
            assert!(notes
                .iter()
                .any(|note| note.contains("is not a member of aggressor alliance")));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn governance_vote_recast_replaces_previous_tally() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);
    open_governance_proposal(&mut world, "proposal.runtime", 6, 1, 5_000);

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.runtime".to_string(),
        option: "approve".to_string(),
        weight: 3,
    });
    world.step().expect("first vote");

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.runtime".to_string(),
        option: "reject".to_string(),
        weight: 1,
    });
    world.step().expect("recast vote");

    let governance = world
        .state()
        .governance_votes
        .get("proposal.runtime")
        .expect("governance vote state");
    assert_eq!(governance.total_weight, 1);
    assert_eq!(governance.tallies.get("reject"), Some(&1_u64));
    assert!(!governance.tallies.contains_key("approve"));
    assert_eq!(governance.votes_by_agent.len(), 1);
}

#[test]
fn governance_proposal_finalizes_and_rejects_late_votes() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c"]);
    open_governance_proposal(&mut world, "proposal.finalize", 2, 3, 6_000);

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "a".to_string(),
        proposal_key: "proposal.finalize".to_string(),
        option: "approve".to_string(),
        weight: 2,
    });
    world.step().expect("vote from a");

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "b".to_string(),
        proposal_key: "proposal.finalize".to_string(),
        option: "approve".to_string(),
        weight: 1,
    });
    world.step().expect("vote from b and finalize");

    let proposal = world
        .state()
        .governance_proposals
        .get("proposal.finalize")
        .expect("finalized proposal");
    assert_eq!(proposal.status, GovernanceProposalStatus::Passed);
    assert_eq!(proposal.winning_option.as_deref(), Some("approve"));
    assert_eq!(proposal.total_weight_at_finalize, 3);

    world.submit_action(Action::CastGovernanceVote {
        voter_agent_id: "c".to_string(),
        proposal_key: "proposal.finalize".to_string(),
        option: "reject".to_string(),
        weight: 5,
    });
    world.step().expect("late vote rejected");

    match last_domain_event(&world) {
        DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        } => {
            assert!(notes
                .iter()
                .any(|note| note.contains("proposal is not open")));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn crisis_cycle_spawns_and_times_out_if_unresolved() {
    let mut world = World::new();
    register_agents(&mut world, &["a"]);
    let crisis_id = advance_until_auto_crisis(&mut world);

    let expires_at = world
        .state()
        .crises
        .get(&crisis_id)
        .expect("active crisis")
        .expires_at;
    while world.state().time <= expires_at {
        world.step().expect("advance to crisis timeout");
    }

    let crisis = world
        .state()
        .crises
        .get(&crisis_id)
        .expect("timed out crisis");
    assert_eq!(crisis.status, CrisisStatus::TimedOut);
    assert_eq!(crisis.success, Some(false));
    assert!(crisis.impact < 0);
    assert!(matches!(
        last_domain_event(&world),
        DomainEvent::CrisisTimedOut { .. }
    ));
}

#[test]
fn war_auto_concludes_after_duration() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c", "d"]);

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "a".to_string(),
        alliance_id: "alliance.red".to_string(),
        members: vec!["b".to_string()],
        charter: "charter.red".to_string(),
    });
    world.step().expect("form red alliance");

    world.submit_action(Action::FormAlliance {
        proposer_agent_id: "c".to_string(),
        alliance_id: "alliance.blue".to_string(),
        members: vec!["d".to_string()],
        charter: "charter.blue".to_string(),
    });
    world.step().expect("form blue alliance");

    world.submit_action(Action::DeclareWar {
        initiator_agent_id: "a".to_string(),
        war_id: "war.auto".to_string(),
        aggressor_alliance_id: "alliance.red".to_string(),
        defender_alliance_id: "alliance.blue".to_string(),
        objective: "hold position".to_string(),
        intensity: 2,
    });
    world.step().expect("declare war");

    for _ in 0..12 {
        world.step().expect("advance war lifecycle");
    }

    let war = world.state().wars.get("war.auto").expect("war state");
    assert!(!war.active);
    assert_eq!(war.winner_alliance_id.as_deref(), Some("alliance.red"));
    assert!(war.concluded_at.is_some());
    assert!(war
        .settlement_summary
        .as_deref()
        .unwrap_or_default()
        .contains("auto settlement"));
}

#[test]
fn meta_progress_unlocks_track_tiers() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);

    world.submit_action(Action::GrantMetaProgress {
        operator_agent_id: "a".to_string(),
        target_agent_id: "b".to_string(),
        track: "campaign".to_string(),
        points: 20,
        achievement_id: None,
    });
    world.step().expect("grant bronze points");

    world.submit_action(Action::GrantMetaProgress {
        operator_agent_id: "a".to_string(),
        target_agent_id: "b".to_string(),
        track: "campaign".to_string(),
        points: 30,
        achievement_id: None,
    });
    world.step().expect("grant silver points");

    world.submit_action(Action::GrantMetaProgress {
        operator_agent_id: "a".to_string(),
        target_agent_id: "b".to_string(),
        track: "campaign".to_string(),
        points: 50,
        achievement_id: None,
    });
    world.step().expect("grant gold points");

    let progress = world.state().meta_progress.get("b").expect("meta progress");
    assert_eq!(progress.track_points.get("campaign"), Some(&100));
    let tiers = progress
        .unlocked_tiers
        .get("campaign")
        .expect("campaign tiers");
    assert!(tiers.iter().any(|tier| tier == "bronze"));
    assert!(tiers.iter().any(|tier| tier == "silver"));
    assert!(tiers.iter().any(|tier| tier == "gold"));
    assert!(progress
        .achievements
        .iter()
        .any(|value| value == "tier.campaign.bronze"));
    assert!(progress
        .achievements
        .iter()
        .any(|value| value == "tier.campaign.silver"));
    assert!(progress
        .achievements
        .iter()
        .any(|value| value == "tier.campaign.gold"));
}

#[test]
fn economic_contract_settlement_applies_tax_and_reputation() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);
    world
        .set_agent_resource_balance("a", ResourceKind::Data, 100)
        .expect("seed creator data");

    world.submit_action(Action::UpdateGameplayPolicy {
        operator_agent_id: "a".to_string(),
        electricity_tax_bps: 0,
        data_tax_bps: 1_000,
        max_open_contracts_per_agent: 4,
        blocked_agents: Vec::new(),
    });
    world.step().expect("update gameplay policy");

    let expires_at = world.state().time.saturating_add(10);
    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.data.1".to_string(),
        counterparty_agent_id: "b".to_string(),
        settlement_kind: ResourceKind::Data,
        settlement_amount: 30,
        reputation_stake: 8,
        expires_at,
        description: "data labeling batch".to_string(),
    });
    world.step().expect("open economic contract");

    world.submit_action(Action::AcceptEconomicContract {
        accepter_agent_id: "b".to_string(),
        contract_id: "contract.data.1".to_string(),
    });
    world.step().expect("accept economic contract");

    world.submit_action(Action::SettleEconomicContract {
        operator_agent_id: "a".to_string(),
        contract_id: "contract.data.1".to_string(),
        success: true,
        notes: "delivered on time".to_string(),
    });
    world.step().expect("settle economic contract");

    let contract = world
        .state()
        .economic_contracts
        .get("contract.data.1")
        .expect("settled contract");
    assert_eq!(contract.status, EconomicContractStatus::Settled);
    assert_eq!(contract.transfer_amount, 30);
    assert_eq!(contract.tax_amount, 3);
    assert_eq!(contract.settlement_success, Some(true));

    let creator_data = world
        .state()
        .agents
        .get("a")
        .expect("creator agent")
        .state
        .resources
        .get(ResourceKind::Data);
    let counterparty_data = world
        .state()
        .agents
        .get("b")
        .expect("counterparty agent")
        .state
        .resources
        .get(ResourceKind::Data);
    assert_eq!(creator_data, 67);
    assert_eq!(counterparty_data, 30);
    assert_eq!(
        world.state().resources.get(&ResourceKind::Data).copied(),
        Some(3)
    );
    assert_eq!(world.state().reputation_scores.get("a"), Some(&8));
    assert_eq!(world.state().reputation_scores.get("b"), Some(&8));
    assert!(matches!(
        last_domain_event(&world),
        DomainEvent::EconomicContractSettled { .. }
    ));
}

#[test]
fn economic_contract_respects_policy_quota_and_block_list() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c"]);

    world.submit_action(Action::UpdateGameplayPolicy {
        operator_agent_id: "a".to_string(),
        electricity_tax_bps: 0,
        data_tax_bps: 0,
        max_open_contracts_per_agent: 1,
        blocked_agents: vec!["b".to_string()],
    });
    world.step().expect("update gameplay policy");

    let expires_at = world.state().time.saturating_add(8);
    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.ok".to_string(),
        counterparty_agent_id: "c".to_string(),
        settlement_kind: ResourceKind::Electricity,
        settlement_amount: 10,
        reputation_stake: 4,
        expires_at,
        description: "power shipment".to_string(),
    });
    world.step().expect("open first contract");

    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.quota".to_string(),
        counterparty_agent_id: "c".to_string(),
        settlement_kind: ResourceKind::Electricity,
        settlement_amount: 10,
        reputation_stake: 4,
        expires_at,
        description: "second contract".to_string(),
    });
    world.step().expect("reject quota overflow");
    match last_domain_event(&world) {
        DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        } => {
            assert!(notes
                .iter()
                .any(|note| note.contains("quota exceeded for creator")));
        }
        other => panic!("unexpected event: {other:?}"),
    }

    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.blocked".to_string(),
        counterparty_agent_id: "b".to_string(),
        settlement_kind: ResourceKind::Electricity,
        settlement_amount: 10,
        reputation_stake: 4,
        expires_at,
        description: "blocked counterparty".to_string(),
    });
    world.step().expect("reject blocked counterparty");
    match last_domain_event(&world) {
        DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        } => {
            assert!(notes
                .iter()
                .any(|note| note.contains("blocked by gameplay policy")));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn economic_contract_expires_and_penalizes_reputation() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);
    let expires_at = world.state().time.saturating_add(2);

    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.expire".to_string(),
        counterparty_agent_id: "b".to_string(),
        settlement_kind: ResourceKind::Electricity,
        settlement_amount: 5,
        reputation_stake: 6,
        expires_at,
        description: "expiring contract".to_string(),
    });
    world.step().expect("open contract");

    while world.state().time <= expires_at {
        world.submit_action(Action::QueryObservation {
            agent_id: "a".to_string(),
        });
        world.step().expect("advance tick for expiry");
    }

    let contract = world
        .state()
        .economic_contracts
        .get("contract.expire")
        .expect("expired contract");
    assert_eq!(contract.status, EconomicContractStatus::Expired);
    assert_eq!(world.state().reputation_scores.get("a"), Some(&-6));
    assert_eq!(world.state().reputation_scores.get("b"), None);
    let has_expired_event = world.journal().events.iter().any(|event| {
        matches!(
            event.body,
            WorldEventBody::Domain(DomainEvent::EconomicContractExpired { .. })
        )
    });
    assert!(has_expired_event, "expected EconomicContractExpired event");
}

#[test]
fn step_with_modules_uses_gameplay_tick_modules_without_fallback() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);
    world
        .install_m5_gameplay_bootstrap_modules("bootstrap")
        .expect("install gameplay modules");

    open_governance_proposal(&mut world, "proposal.module_only", 1, 1, 5_000);

    let mut sandbox = GameplayDirectiveSandbox::empty();
    world
        .step_with_modules(&mut sandbox)
        .expect("module-driven gameplay tick");

    let proposal = world
        .state()
        .governance_proposals
        .get("proposal.module_only")
        .expect("proposal should still exist");
    assert_eq!(proposal.status, GovernanceProposalStatus::Open);
}

#[test]
fn step_with_modules_applies_gameplay_directive_emits_to_domain_events() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);
    world
        .install_m5_gameplay_bootstrap_modules("bootstrap")
        .expect("install gameplay modules");

    open_governance_proposal(&mut world, "proposal.directive", 8, 1, 5_000);

    let payload = serde_json::json!({
        "directives": [
            {
                "type": "governance_finalize",
                "proposal_key": "proposal.directive",
                "winning_option": "approve",
                "winning_weight": 3,
                "total_weight": 3,
                "passed": true
            }
        ]
    });
    let mut sandbox = GameplayDirectiveSandbox::with_governance_directive(payload);
    world
        .step_with_modules(&mut sandbox)
        .expect("module-driven directive tick");

    let proposal = world
        .state()
        .governance_proposals
        .get("proposal.directive")
        .expect("proposal finalized by directive");
    assert_eq!(proposal.status, GovernanceProposalStatus::Passed);
    assert_eq!(proposal.winning_option.as_deref(), Some("approve"));
    assert_eq!(proposal.winning_weight, 3);
    assert_eq!(proposal.total_weight_at_finalize, 3);
}
