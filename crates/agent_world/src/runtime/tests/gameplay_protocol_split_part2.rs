#[test]
fn economic_contract_settlement_overflow_keeps_state_atomic() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b"]);
    authorize_policy_update(&mut world, "a", "proposal.policy.atomicity");
    world
        .set_agent_resource_balance("a", ResourceKind::Data, 100)
        .expect("seed creator data");
    world
        .set_agent_resource_balance("b", ResourceKind::Data, i64::MAX)
        .expect("seed counterparty data at boundary");

    world.submit_action(Action::UpdateGameplayPolicy {
        operator_agent_id: "a".to_string(),
        electricity_tax_bps: 0,
        data_tax_bps: 0,
        max_open_contracts_per_agent: 4,
        blocked_agents: Vec::new(),
    });
    world.step().expect("update gameplay policy");

    let expires_at = world.state().time.saturating_add(10);
    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.atomic.overflow".to_string(),
        counterparty_agent_id: "b".to_string(),
        settlement_kind: ResourceKind::Data,
        settlement_amount: 1,
        reputation_stake: 5,
        expires_at,
        description: "overflow settlement".to_string(),
    });
    world.step().expect("open economic contract");
    world.submit_action(Action::AcceptEconomicContract {
        accepter_agent_id: "b".to_string(),
        contract_id: "contract.atomic.overflow".to_string(),
    });
    world.step().expect("accept economic contract");
    let events_before = world.journal().len();

    world.submit_action(Action::SettleEconomicContract {
        operator_agent_id: "a".to_string(),
        contract_id: "contract.atomic.overflow".to_string(),
        success: true,
        notes: "attempt overflow settle".to_string(),
    });
    let err = world.step().expect_err("overflow settlement must fail");
    assert!(
        matches!(err, WorldError::ResourceBalanceInvalid { .. }),
        "unexpected error: {err:?}"
    );

    let contract = world
        .state()
        .economic_contracts
        .get("contract.atomic.overflow")
        .expect("contract should still exist");
    assert_eq!(contract.status, EconomicContractStatus::Accepted);
    assert_eq!(contract.settled_at, None);
    assert_eq!(contract.settlement_success, None);
    assert_eq!(contract.transfer_amount, 0);
    assert_eq!(contract.tax_amount, 0);
    assert_eq!(contract.settlement_notes, None);
    assert_eq!(
        world
            .state()
            .agents
            .get("a")
            .expect("creator agent")
            .state
            .resources
            .get(ResourceKind::Data),
        100
    );
    assert_eq!(
        world
            .state()
            .agents
            .get("b")
            .expect("counterparty agent")
            .state
            .resources
            .get(ResourceKind::Data),
        i64::MAX
    );
    assert_eq!(
        world.state().resources.get(&ResourceKind::Data).copied(),
        None
    );
    assert_eq!(world.state().reputation_scores.get("a"), None);
    assert_eq!(world.state().reputation_scores.get("b"), None);
    assert_eq!(world.journal().len(), events_before);
}

#[test]
fn economic_contract_success_reputation_reward_respects_stake_and_cap() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c"]);
    authorize_policy_update(&mut world, "a", "proposal.policy.reputation");
    world
        .set_agent_resource_balance("a", ResourceKind::Data, 2_000)
        .expect("seed creator data");

    world.submit_action(Action::UpdateGameplayPolicy {
        operator_agent_id: "a".to_string(),
        electricity_tax_bps: 0,
        data_tax_bps: 0,
        max_open_contracts_per_agent: 8,
        blocked_agents: Vec::new(),
    });
    world.step().expect("update gameplay policy");

    let expires_at = world.state().time.saturating_add(10);
    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.stake.bound".to_string(),
        counterparty_agent_id: "b".to_string(),
        settlement_kind: ResourceKind::Data,
        settlement_amount: 200,
        reputation_stake: 5,
        expires_at,
        description: "stake bound contract".to_string(),
    });
    world.step().expect("open stake-bound contract");
    world.submit_action(Action::AcceptEconomicContract {
        accepter_agent_id: "b".to_string(),
        contract_id: "contract.stake.bound".to_string(),
    });
    world.step().expect("accept stake-bound contract");
    world.submit_action(Action::SettleEconomicContract {
        operator_agent_id: "a".to_string(),
        contract_id: "contract.stake.bound".to_string(),
        success: true,
        notes: "settle stake bound".to_string(),
    });
    world.step().expect("settle stake-bound contract");

    let second_expires_at = world.state().time.saturating_add(10);
    world.submit_action(Action::OpenEconomicContract {
        creator_agent_id: "a".to_string(),
        contract_id: "contract.cap.bound".to_string(),
        counterparty_agent_id: "c".to_string(),
        settlement_kind: ResourceKind::Data,
        settlement_amount: 500,
        reputation_stake: 80,
        expires_at: second_expires_at,
        description: "cap bound contract".to_string(),
    });
    world.step().expect("open cap-bound contract");
    world.submit_action(Action::AcceptEconomicContract {
        accepter_agent_id: "c".to_string(),
        contract_id: "contract.cap.bound".to_string(),
    });
    world.step().expect("accept cap-bound contract");
    world.submit_action(Action::SettleEconomicContract {
        operator_agent_id: "a".to_string(),
        contract_id: "contract.cap.bound".to_string(),
        success: true,
        notes: "settle cap bound".to_string(),
    });
    world.step().expect("settle cap-bound contract");

    assert_eq!(world.state().reputation_scores.get("a"), Some(&17));
    assert_eq!(world.state().reputation_scores.get("b"), Some(&5));
    assert_eq!(world.state().reputation_scores.get("c"), Some(&12));
}

#[test]
fn economic_contract_respects_policy_quota_and_block_list() {
    let mut world = World::new();
    register_agents(&mut world, &["a", "b", "c"]);
    authorize_policy_update(&mut world, "a", "proposal.policy.quota");

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
    assert_latest_rule_denied_contains(&world, "quota exceeded for creator");

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
    assert_latest_rule_denied_contains(&world, "blocked by gameplay policy");
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
