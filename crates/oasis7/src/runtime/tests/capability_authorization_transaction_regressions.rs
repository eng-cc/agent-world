use super::super::*;
use oasis7_wasm_abi::{CapabilityAudience, CapabilityPresenter, CapabilitySubject};
use serde_json::json;

#[test]
fn capability_invocation_context_publication_failure_is_fully_unpublished() {
    let mut world = super::capability_grant_v2::fixture_world();
    let context = CapabilityInvocationContext {
        grant_id: "grant-context-publication-regression".to_string(),
        subject: CapabilitySubject::System {
            system_id: "system-context-publication-regression".to_string(),
            epoch: 0,
        },
        presenter: CapabilityPresenter {
            presenter_id: "provider-context-publication-regression".to_string(),
            presenter_kind: "provider".to_string(),
            session_id: Some("session-context-publication-regression".to_string()),
            attestation_ref: None,
        },
        audience: CapabilityAudience {
            world_id: "world.context-publication-regression".to_string(),
            branch_id: "branch-context-publication-regression".to_string(),
            finality_epoch: 0,
            target_kind: "world".to_string(),
            target_id: None,
        },
        catalog_snapshot_id: "catalog-context-publication-regression".to_string(),
        module_id: "module.context-publication-regression".to_string(),
        module_version: "1.0.0".to_string(),
        response_nonce: "response-context-publication-regression".to_string(),
    };

    let snapshot_before = world.snapshot();
    let journal_before = world.journal().clone();
    let contexts_before = world.capability_invocation_contexts().clone();
    let system_identities_before = world
        .capability_revocation_state()
        .system_identities
        .clone();
    let authorization_root_before = world.capability_authorization_root().to_string();
    let consensus_before = world.tick_consensus_records().to_vec();
    let rejection_audit_before = world.tick_consensus_rejection_audit_events().to_vec();
    let backpressure_before = world.runtime_backpressure_stats().clone();
    let mut expected_world = world.clone();

    // The System subject must publish two authorization events. The failpoint
    // is after publication preparation, so neither event may install a
    // system identity, context, event id, journal entry, consensus record, or
    // backpressure update before the runtime transaction commits atomically.
    world.fail_next_append_after_publication_prepare_for_test();
    let error = world
        .install_capability_invocation_context(context.clone())
        .expect_err("post-prepare failure must abort public context installation");
    assert!(matches!(
        error,
        WorldError::ResourceBalanceInvalid { ref reason }
            if reason.contains("publication preparation")
    ));

    assert_eq!(world.snapshot(), snapshot_before);
    assert_eq!(world.journal(), &journal_before);
    assert_eq!(
        world.snapshot().last_event_id,
        snapshot_before.last_event_id
    );
    assert_eq!(world.snapshot().event_id_era, snapshot_before.event_id_era);
    assert_eq!(world.capability_invocation_contexts(), &contexts_before);
    assert_eq!(
        world.capability_revocation_state().system_identities,
        system_identities_before
    );
    assert_eq!(
        world.capability_authorization_root(),
        authorization_root_before
    );
    assert_eq!(world.tick_consensus_records(), consensus_before.as_slice());
    assert_eq!(
        world.tick_consensus_rejection_audit_events(),
        rejection_audit_before.as_slice()
    );
    assert_eq!(world.runtime_backpressure_stats(), &backpressure_before);

    expected_world
        .install_capability_invocation_context(context.clone())
        .expect("control context installation");
    world
        .install_capability_invocation_context(context)
        .expect("retry context installation after one-shot failpoint");

    assert_eq!(world.snapshot(), expected_world.snapshot());
    assert_eq!(world.journal(), expected_world.journal());
    assert_eq!(
        world.capability_invocation_contexts(),
        expected_world.capability_invocation_contexts()
    );
    assert_eq!(
        world.capability_revocation_state().system_identities,
        expected_world
            .capability_revocation_state()
            .system_identities
    );
    assert_eq!(
        world.capability_authorization_root(),
        expected_world.capability_authorization_root()
    );
    assert_eq!(
        world.tick_consensus_records(),
        expected_world.tick_consensus_records()
    );
    assert_eq!(
        world.tick_consensus_rejection_audit_events(),
        expected_world.tick_consensus_rejection_audit_events()
    );
    assert_eq!(
        world.runtime_backpressure_stats(),
        expected_world.runtime_backpressure_stats()
    );

    let tail = &world.journal().events[journal_before.events.len()..];
    assert_eq!(
        tail.len(),
        2,
        "System context installation must publish two events"
    );
    assert!(matches!(
        &tail[0].body,
        WorldEventBody::CapabilityAuthorization(
            CapabilityAuthorizationEvent::SystemIdentityInstalled { system_id, epoch }
        ) if system_id == "system-context-publication-regression" && *epoch == 0
    ));
    assert!(matches!(
        &tail[1].body,
        WorldEventBody::CapabilityAuthorization(
            CapabilityAuthorizationEvent::InvocationContextInstalled { context: installed, .. }
        ) if installed.grant_id == "grant-context-publication-regression"
            && installed.response_nonce == "response-context-publication-regression"
            && matches!(
                &installed.subject,
                CapabilitySubject::System { system_id, epoch }
                    if system_id == "system-context-publication-regression" && *epoch == 0
            )
    ));

    let replayed = World::from_snapshot(snapshot_before, world.journal().clone())
        .expect("replay successful context installation");
    assert_eq!(replayed.state(), world.state());
    assert_eq!(replayed.journal(), world.journal());
    assert_eq!(
        replayed.snapshot().last_event_id,
        world.snapshot().last_event_id
    );
    assert_eq!(
        replayed.snapshot().event_id_era,
        world.snapshot().event_id_era
    );
    assert_eq!(
        replayed.capability_invocation_contexts(),
        world.capability_invocation_contexts()
    );
    assert_eq!(
        replayed.capability_revocation_state().system_identities,
        world.capability_revocation_state().system_identities
    );
    assert_eq!(
        replayed.capability_authorization_root(),
        world.capability_authorization_root()
    );
    assert_eq!(
        replayed.tick_consensus_records(),
        world.tick_consensus_records()
    );
    assert_eq!(
        replayed.tick_consensus_rejection_audit_events(),
        world.tick_consensus_rejection_audit_events()
    );
    assert_eq!(
        replayed.runtime_backpressure_stats(),
        world.runtime_backpressure_stats()
    );
}

#[test]
fn capability_budget_account_publication_failure_is_fully_unpublished() {
    let mut world = super::capability_grant_v2::fixture_world();
    let subject = world
        .capability_budget_accounts()
        .values()
        .next()
        .expect("fixture has a valid budget subject")
        .subject
        .clone();
    let account = CapabilityBudgetAccount {
        subject,
        grant_id: "grant-budget-account-publication-regression".to_string(),
        remaining_units: 64,
        reserved_units: 0,
        spent_units: 0,
    };

    let snapshot_before = world.snapshot();
    let journal_before = world.journal().clone();
    let event_id_before = snapshot_before.last_event_id;
    let event_id_era_before = snapshot_before.event_id_era;
    let budget_accounts_before = world.capability_budget_accounts().clone();
    let authorization_root_before = world.capability_authorization_root().to_string();
    let consensus_before = world.tick_consensus_records().to_vec();
    let rejection_audit_before = world.tick_consensus_rejection_audit_events().to_vec();
    let backpressure_before = world.runtime_backpressure_stats().clone();
    let mut expected_world = world.clone();

    // The public budget installer still routes through the legacy append path.
    // The injected post-prepare failure must therefore abort before the budget
    // account, event id, journal, authorization root, consensus, or
    // deterministic backpressure can become observable.
    world.fail_next_append_after_publication_prepare_for_test();
    let error = world
        .install_capability_budget_account(account.clone())
        .expect_err("post-prepare failure must abort public budget installation");
    assert!(matches!(
        error,
        WorldError::ResourceBalanceInvalid { ref reason }
            if reason.contains("publication preparation")
    ));

    assert_eq!(world.snapshot(), snapshot_before);
    assert_eq!(world.journal(), &journal_before);
    assert_eq!(world.snapshot().last_event_id, event_id_before);
    assert_eq!(world.snapshot().event_id_era, event_id_era_before);
    assert_eq!(world.capability_budget_accounts(), &budget_accounts_before);
    assert_eq!(
        world.capability_authorization_root(),
        authorization_root_before
    );
    assert_eq!(world.tick_consensus_records(), consensus_before.as_slice());
    assert_eq!(
        world.tick_consensus_rejection_audit_events(),
        rejection_audit_before.as_slice()
    );
    assert_eq!(world.runtime_backpressure_stats(), &backpressure_before);

    expected_world
        .install_capability_budget_account(account.clone())
        .expect("control budget account installation");
    world
        .install_capability_budget_account(account.clone())
        .expect("retry budget account installation after one-shot failpoint");

    assert_eq!(world.snapshot(), expected_world.snapshot());
    assert_eq!(world.journal(), expected_world.journal());
    assert_eq!(
        world.snapshot().last_event_id,
        expected_world.snapshot().last_event_id
    );
    assert_eq!(
        world.snapshot().event_id_era,
        expected_world.snapshot().event_id_era
    );
    assert_eq!(
        world.capability_budget_accounts(),
        expected_world.capability_budget_accounts()
    );
    assert_eq!(
        world.capability_authorization_root(),
        expected_world.capability_authorization_root()
    );
    assert_eq!(
        world.tick_consensus_records(),
        expected_world.tick_consensus_records()
    );
    assert_eq!(
        world.tick_consensus_rejection_audit_events(),
        expected_world.tick_consensus_rejection_audit_events()
    );
    assert_eq!(
        world.runtime_backpressure_stats(),
        expected_world.runtime_backpressure_stats()
    );

    let tail = &world.journal().events[journal_before.events.len()..];
    assert_eq!(
        tail.len(),
        1,
        "budget account installation must publish one event"
    );
    assert!(matches!(
        &tail[0].body,
        WorldEventBody::CapabilityAuthorization(
            CapabilityAuthorizationEvent::BudgetAccountInstalled {
                account: installed,
                ..
            }
        ) if installed == &account
    ));

    let replayed = World::from_snapshot(snapshot_before, world.journal().clone())
        .expect("replay successful budget account installation");
    assert_eq!(replayed.state(), world.state());
    assert_eq!(replayed.journal(), world.journal());
    assert_eq!(
        replayed.snapshot().last_event_id,
        world.snapshot().last_event_id
    );
    assert_eq!(
        replayed.snapshot().event_id_era,
        world.snapshot().event_id_era
    );
    assert_eq!(
        replayed.capability_budget_accounts(),
        world.capability_budget_accounts()
    );
    assert_eq!(
        replayed.capability_authorization_root(),
        world.capability_authorization_root()
    );
    assert_eq!(
        replayed.tick_consensus_records(),
        world.tick_consensus_records()
    );
    assert_eq!(
        replayed.tick_consensus_rejection_audit_events(),
        world.tick_consensus_rejection_audit_events()
    );
    assert_eq!(
        replayed.runtime_backpressure_stats(),
        world.runtime_backpressure_stats()
    );
}

#[test]
fn capability_grant_registration_publication_failure_is_fully_unpublished() {
    let mut world = super::capability_grant_v2::fixture_world();
    let grant =
        super::capability_grant_v2::signed_grant(super::capability_grant_v2::grant_json(json!({
            "grant_nonce": "grant-registration-publication-regression"
        })));

    let snapshot_before = world.snapshot();
    let journal_before = world.journal().clone();
    let event_id_before = snapshot_before.last_event_id;
    let event_id_era_before = snapshot_before.event_id_era;
    let grants_before = world.capability_grants_v2().clone();
    let authorization_root_before = world.capability_authorization_root().to_string();
    let consensus_before = world.tick_consensus_records().to_vec();
    let rejection_audit_before = world.tick_consensus_rejection_audit_events().to_vec();
    let backpressure_before = world.runtime_backpressure_stats().clone();
    let mut expected_world = world.clone();

    // The public grant-registration API still routes through the legacy
    // append path. A post-prepare failure must leave every durable grant and
    // publication projection untouched until the typed transaction commits.
    world.fail_next_append_after_publication_prepare_for_test();
    let error = world
        .register_capability_grant_v2(grant.clone())
        .expect_err("post-prepare failure must abort public grant registration");
    assert!(matches!(
        error,
        WorldError::ResourceBalanceInvalid { ref reason }
            if reason.contains("publication preparation")
    ));

    assert_eq!(world.snapshot(), snapshot_before);
    assert_eq!(world.journal(), &journal_before);
    assert_eq!(world.snapshot().last_event_id, event_id_before);
    assert_eq!(world.snapshot().event_id_era, event_id_era_before);
    assert_eq!(world.capability_grants_v2(), &grants_before);
    assert_eq!(
        world.capability_authorization_root(),
        authorization_root_before
    );
    assert_eq!(world.tick_consensus_records(), consensus_before.as_slice());
    assert_eq!(
        world.tick_consensus_rejection_audit_events(),
        rejection_audit_before.as_slice()
    );
    assert_eq!(world.runtime_backpressure_stats(), &backpressure_before);

    expected_world
        .register_capability_grant_v2(grant.clone())
        .expect("control grant registration");
    world
        .register_capability_grant_v2(grant.clone())
        .expect("retry grant registration after one-shot failpoint");

    assert_eq!(world.snapshot(), expected_world.snapshot());
    assert_eq!(world.journal(), expected_world.journal());
    assert_eq!(
        world.snapshot().last_event_id,
        expected_world.snapshot().last_event_id
    );
    assert_eq!(
        world.snapshot().event_id_era,
        expected_world.snapshot().event_id_era
    );
    assert_eq!(
        world.capability_grants_v2(),
        expected_world.capability_grants_v2()
    );
    assert_eq!(
        world.capability_authorization_root(),
        expected_world.capability_authorization_root()
    );
    assert_eq!(
        world.tick_consensus_records(),
        expected_world.tick_consensus_records()
    );
    assert_eq!(
        world.tick_consensus_rejection_audit_events(),
        expected_world.tick_consensus_rejection_audit_events()
    );
    assert_eq!(
        world.runtime_backpressure_stats(),
        expected_world.runtime_backpressure_stats()
    );

    let tail = &world.journal().events[journal_before.events.len()..];
    assert_eq!(
        tail.len(),
        1,
        "grant registration must publish one authorization event"
    );
    assert!(matches!(
        &tail[0].body,
        WorldEventBody::CapabilityAuthorization(
            CapabilityAuthorizationEvent::GrantRegistered { grant: installed }
        ) if installed == &grant
    ));

    let replayed = World::from_snapshot(snapshot_before, world.journal().clone())
        .expect("replay successful grant registration");
    assert_eq!(replayed.state(), world.state());
    assert_eq!(replayed.journal(), world.journal());
    assert_eq!(
        replayed.snapshot().last_event_id,
        world.snapshot().last_event_id
    );
    assert_eq!(
        replayed.snapshot().event_id_era,
        world.snapshot().event_id_era
    );
    assert_eq!(
        replayed.capability_grants_v2(),
        world.capability_grants_v2()
    );
    assert_eq!(
        replayed.capability_authorization_root(),
        world.capability_authorization_root()
    );
    assert_eq!(
        replayed.tick_consensus_records(),
        world.tick_consensus_records()
    );
    assert_eq!(
        replayed.tick_consensus_rejection_audit_events(),
        world.tick_consensus_rejection_audit_events()
    );
    assert_eq!(
        replayed.runtime_backpressure_stats(),
        world.runtime_backpressure_stats()
    );
}
