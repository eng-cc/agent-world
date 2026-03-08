use super::super::*;
use super::pos;
use serde_json::json;

fn local_guardians() -> Vec<String> {
    vec![
        "governance.local.finality.signer.1".to_string(),
        "governance.local.finality.signer.2".to_string(),
    ]
}

fn register_agent(world: &mut World, agent_id: &str, x: f64, y: f64) {
    world.submit_action(Action::RegisterAgent {
        agent_id: agent_id.to_string(),
        pos: pos(x, y),
    });
    world.step().unwrap();
}

#[test]
fn governance_flow_applies_manifest() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "demo" }),
    };

    let proposal_id = world
        .propose_manifest_update(manifest.clone(), "alice")
        .unwrap();
    let shadow_hash = world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    let applied_hash = world.apply_proposal(proposal_id).unwrap();

    assert_eq!(shadow_hash, applied_hash);
    assert_eq!(world.manifest().version, 2);
    assert_eq!(world.manifest().content, manifest.content);
}

#[test]
fn governance_policy_blocks_local_apply_proposal_path() {
    let mut world = World::new();
    world.enable_production_release_policy();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "external-finality-only" }),
    };

    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    let err = world.apply_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::GovernancePolicyInvalid { .. }));

    let certificate = world.build_local_finality_certificate(proposal_id).unwrap();
    world
        .apply_proposal_with_finality(proposal_id, &certificate)
        .expect("apply with explicit finality cert");
}

#[test]
fn governance_patch_updates_manifest() {
    let mut world = World::new();
    let base_hash = world.current_manifest_hash().unwrap();
    let patch = ManifestPatch {
        base_manifest_hash: base_hash,
        ops: vec![ManifestPatchOp::Set {
            path: vec!["settings".to_string(), "mode".to_string()],
            value: json!("fast"),
        }],
        new_version: Some(3),
    };

    let proposal_id = world.propose_manifest_patch(patch, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();

    assert_eq!(world.manifest().version, 3);
    assert_eq!(
        world.manifest().content,
        json!({ "settings": { "mode": "fast" } })
    );
}

#[test]
fn manifest_diff_and_merge() {
    let base = Manifest {
        version: 1,
        content: json!({ "a": 1, "b": { "c": 2 } }),
    };
    let target = Manifest {
        version: 2,
        content: json!({ "a": 1, "b": { "c": 3 }, "d": 4 }),
    };

    let patch = diff_manifest(&base, &target).unwrap();
    let applied = apply_manifest_patch(&base, &patch).unwrap();
    assert_eq!(applied, target);

    let base_hash = util::hash_json(&base).unwrap();
    let patch1 = ManifestPatch {
        base_manifest_hash: base_hash.clone(),
        ops: vec![ManifestPatchOp::Set {
            path: vec!["b".to_string(), "c".to_string()],
            value: json!(3),
        }],
        new_version: Some(2),
    };
    let patch2 = ManifestPatch {
        base_manifest_hash: base_hash,
        ops: vec![ManifestPatchOp::Set {
            path: vec!["e".to_string()],
            value: json!(5),
        }],
        new_version: Some(3),
    };

    let merged = merge_manifest_patches(&base, &[patch1, patch2]).unwrap();
    let merged_applied = apply_manifest_patch(&base, &merged).unwrap();
    let expected = Manifest {
        version: 3,
        content: json!({ "a": 1, "b": { "c": 3 }, "e": 5 }),
    };
    assert_eq!(merged_applied, expected);
}

#[test]
fn merge_reports_conflicts() {
    let base = Manifest {
        version: 1,
        content: json!({ "a": { "b": 1 }, "x": 1 }),
    };
    let base_hash = util::hash_json(&base).unwrap();
    let patch1 = ManifestPatch {
        base_manifest_hash: base_hash.clone(),
        ops: vec![ManifestPatchOp::Set {
            path: vec!["a".to_string(), "b".to_string()],
            value: json!(2),
        }],
        new_version: None,
    };
    let patch2 = ManifestPatch {
        base_manifest_hash: base_hash,
        ops: vec![ManifestPatchOp::Set {
            path: vec!["a".to_string()],
            value: json!({ "b": 3 }),
        }],
        new_version: None,
    };

    let result = merge_manifest_patches_with_conflicts(&base, &[patch1, patch2]).unwrap();
    assert_eq!(result.conflicts.len(), 1);
    assert_eq!(result.conflicts[0].path, vec!["a".to_string()]);
    assert_eq!(result.conflicts[0].kind, ConflictKind::PrefixOverlap);
    assert_eq!(result.conflicts[0].patches, vec![0, 1]);
    assert_eq!(result.conflicts[0].ops.len(), 2);
}

#[test]
fn governance_apply_with_finality_rejects_threshold_mismatch() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "demo" }),
    };
    let proposal_id = world
        .propose_manifest_update(manifest.clone(), "alice")
        .unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    let mut certificate = world.build_local_finality_certificate(proposal_id).unwrap();
    certificate.threshold = certificate.threshold.saturating_add(1);
    let err = world
        .apply_proposal_with_finality(proposal_id, &certificate)
        .unwrap_err();
    assert!(matches!(err, WorldError::GovernanceFinalityInvalid { .. }));
}

#[test]
fn governance_apply_emits_manifest_updated_before_applied() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "demo" }),
    };
    let proposal_id = world
        .propose_manifest_update(manifest.clone(), "alice")
        .unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();
    let certificate = world.build_local_finality_certificate(proposal_id).unwrap();

    world
        .apply_proposal_with_finality(proposal_id, &certificate)
        .unwrap();

    let mut manifest_updated_idx = None;
    let mut applied_idx = None;
    for (idx, event) in world.journal().events.iter().enumerate() {
        match &event.body {
            WorldEventBody::ManifestUpdated(_) => manifest_updated_idx = Some(idx),
            WorldEventBody::Governance(GovernanceEvent::Applied {
                proposal_id: pid, ..
            }) if *pid == proposal_id => applied_idx = Some(idx),
            _ => {}
        }
    }
    let manifest_updated_idx = manifest_updated_idx.expect("manifest updated event");
    let applied_idx = applied_idx.expect("applied event");
    assert!(
        manifest_updated_idx < applied_idx,
        "manifest must be updated before applied marker"
    );
}

#[test]
fn governance_timelock_blocks_early_apply() {
    let mut world = World::new();
    world
        .set_governance_execution_policy(GovernanceExecutionPolicy {
            timelock_ticks: 3,
            ..GovernanceExecutionPolicy::default()
        })
        .unwrap();

    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "timelock" }),
    };
    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    let err = world.apply_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::GovernancePolicyInvalid { .. }));

    for _ in 0..3 {
        world.step().unwrap();
    }
    world.apply_proposal(proposal_id).unwrap();
}

#[test]
fn governance_epoch_gate_blocks_early_apply() {
    let mut world = World::new();
    world
        .set_governance_execution_policy(GovernanceExecutionPolicy {
            epoch_length_ticks: 5,
            activation_delay_epochs: 1,
            ..GovernanceExecutionPolicy::default()
        })
        .unwrap();

    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "epoch" }),
    };
    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    let err = world.apply_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::GovernancePolicyInvalid { .. }));

    for _ in 0..5 {
        world.step().unwrap();
    }
    world.apply_proposal(proposal_id).unwrap();
}

#[test]
fn governance_emergency_brake_and_release_gate_apply() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "brake" }),
    };
    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    world
        .activate_emergency_brake("guardian-1", "incident", 4, local_guardians())
        .unwrap();
    let err = world.apply_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::GovernancePolicyInvalid { .. }));

    world
        .release_emergency_brake("guardian-2", "incident mitigated", local_guardians())
        .unwrap();
    world.apply_proposal(proposal_id).unwrap();
}

#[test]
fn governance_emergency_veto_rejects_queued_proposal() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "veto" }),
    };
    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    world
        .emergency_veto_proposal(
            proposal_id,
            "guardian-1",
            "unsafe parameter drift",
            local_guardians(),
        )
        .unwrap();
    let proposal = world.proposals().get(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Rejected { .. }));
    let ProposalStatus::Rejected { reason } = &proposal.status else {
        panic!("proposal should be rejected");
    };
    assert!(reason.contains("emergency_veto"));

    let err = world.apply_proposal(proposal_id).unwrap_err();
    assert!(matches!(err, WorldError::ProposalInvalidState { .. }));
}

#[test]
fn governance_emergency_controls_reject_invalid_guardian_signatures() {
    let mut world = World::new();
    let manifest = Manifest {
        version: 2,
        content: json!({ "name": "guardian-check" }),
    };
    let proposal_id = world.propose_manifest_update(manifest, "alice").unwrap();
    world.shadow_proposal(proposal_id).unwrap();
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .unwrap();

    let below_threshold = world
        .activate_emergency_brake(
            "guardian-1",
            "threshold check",
            3,
            vec!["governance.local.finality.signer.1".to_string()],
        )
        .unwrap_err();
    assert!(matches!(
        below_threshold,
        WorldError::GovernancePolicyInvalid { .. }
    ));

    let untrusted_signer = world
        .emergency_veto_proposal(
            proposal_id,
            "guardian-2",
            "untrusted signer",
            vec![
                "governance.local.finality.signer.1".to_string(),
                "governance.unknown.signer".to_string(),
            ],
        )
        .unwrap_err();
    assert!(matches!(
        untrusted_signer,
        WorldError::GovernancePolicyInvalid { .. }
    ));
}

#[test]
fn governance_identity_penalty_freezes_and_slashes_profile() {
    let mut world = World::new();
    register_agent(&mut world, "agent-1", 0.0, 0.0);
    world
        .set_governance_identity_profile("agent-1", 100, 0, GovernanceIdentityStatus::Active)
        .unwrap();

    let penalty_id = world
        .apply_identity_penalty(
            "agent-1",
            "evidence.sybil.cluster",
            "suspected sybil coordination",
            40,
            10,
            "guardian-1",
            local_guardians(),
        )
        .unwrap();

    let profile = world.governance_identity_profile("agent-1").unwrap();
    assert_eq!(profile.status, GovernanceIdentityStatus::Frozen);
    assert_eq!(profile.stake_locked, 60);
    assert_eq!(profile.slash_count, 1);

    let record = world
        .governance_identity_penalties()
        .get(&penalty_id)
        .unwrap();
    assert_eq!(record.status, GovernanceIdentityPenaltyStatus::Applied);
    assert_eq!(record.slash_stake, 40);
    assert_eq!(record.target_agent_id, "agent-1");
}

#[test]
fn governance_identity_penalty_appeal_accept_restores_profile() {
    let mut world = World::new();
    register_agent(&mut world, "agent-1", 0.0, 0.0);
    world
        .set_governance_identity_profile("agent-1", 50, 0, GovernanceIdentityStatus::Active)
        .unwrap();

    let penalty_id = world
        .apply_identity_penalty(
            "agent-1",
            "evidence.fp.case",
            "potential false positive",
            20,
            10,
            "guardian-1",
            local_guardians(),
        )
        .unwrap();
    world
        .appeal_identity_penalty(penalty_id, "agent-1", "request review")
        .unwrap();
    world
        .resolve_identity_penalty_appeal(penalty_id, "committee", true, "appeal accepted")
        .unwrap();

    let profile = world.governance_identity_profile("agent-1").unwrap();
    assert_eq!(profile.status, GovernanceIdentityStatus::Active);
    assert_eq!(profile.stake_locked, 50);

    let record = world
        .governance_identity_penalties()
        .get(&penalty_id)
        .unwrap();
    assert_eq!(
        record.status,
        GovernanceIdentityPenaltyStatus::AppealAccepted
    );
    assert_eq!(record.resolved_by.as_deref(), Some("committee"));
}

#[test]
fn governance_identity_penalty_appeal_respects_deadline() {
    let mut world = World::new();
    register_agent(&mut world, "agent-1", 0.0, 0.0);
    world
        .set_governance_identity_profile("agent-1", 30, 0, GovernanceIdentityStatus::Active)
        .unwrap();
    let penalty_id = world
        .apply_identity_penalty(
            "agent-1",
            "evidence.deadline.case",
            "deadline check",
            10,
            1,
            "guardian-1",
            local_guardians(),
        )
        .unwrap();

    for _ in 0..2 {
        world.step().unwrap();
    }
    let err = world
        .appeal_identity_penalty(penalty_id, "agent-1", "too late")
        .unwrap_err();
    assert!(matches!(err, WorldError::GovernancePolicyInvalid { .. }));
}

#[test]
fn governance_identity_penalty_rejects_duplicate_incident_signature() {
    let mut world = World::new();
    register_agent(&mut world, "agent-1", 0.0, 0.0);
    world
        .set_governance_identity_profile("agent-1", 80, 0, GovernanceIdentityStatus::Active)
        .unwrap();

    world
        .apply_identity_penalty(
            "agent-1",
            "evidence.sybil.replay",
            "first signal",
            10,
            10,
            "guardian-1",
            local_guardians(),
        )
        .unwrap();
    let err = world
        .apply_identity_penalty(
            "agent-1",
            "evidence.sybil.replay",
            "duplicate signal",
            10,
            10,
            "guardian-1",
            local_guardians(),
        )
        .unwrap_err();
    assert!(matches!(err, WorldError::GovernancePolicyInvalid { .. }));
    let WorldError::GovernancePolicyInvalid { reason } = err else {
        panic!("expected governance policy invalid");
    };
    assert!(reason.contains("duplicate identity penalty incident"));
}

#[test]
fn governance_identity_penalty_evidence_chain_tracks_appeal_and_resolution() {
    let mut world = World::new();
    register_agent(&mut world, "agent-1", 0.0, 0.0);
    world
        .set_governance_identity_profile("agent-1", 60, 0, GovernanceIdentityStatus::Active)
        .unwrap();

    let penalty_id = world
        .apply_identity_penalty(
            "agent-1",
            "evidence.sybil.chain",
            "chain seed",
            20,
            10,
            "guardian-1",
            local_guardians(),
        )
        .unwrap();
    let root_chain_hash = world
        .governance_identity_penalties()
        .get(&penalty_id)
        .unwrap()
        .evidence_chain_hash
        .clone();
    assert!(!root_chain_hash.is_empty());

    world
        .appeal_identity_penalty(penalty_id, "agent-1", "provide counter evidence")
        .unwrap();
    let appealed = world
        .governance_identity_penalties()
        .get(&penalty_id)
        .unwrap();
    assert!(appealed.appeal_evidence_hash.is_some());
    assert_ne!(appealed.evidence_chain_hash, root_chain_hash);
    let appeal_chain_hash = appealed.evidence_chain_hash.clone();

    world
        .resolve_identity_penalty_appeal(penalty_id, "committee", false, "appeal rejected")
        .unwrap();
    let resolved = world
        .governance_identity_penalties()
        .get(&penalty_id)
        .unwrap();
    assert!(resolved.resolution_evidence_hash.is_some());
    assert_ne!(resolved.evidence_chain_hash, appeal_chain_hash);
}

#[test]
fn governance_identity_penalty_monitor_reports_false_positive_and_open_risk() {
    let mut world = World::new();
    register_agent(&mut world, "agent-1", 0.0, 0.0);
    world
        .set_governance_identity_profile("agent-1", 100, 0, GovernanceIdentityStatus::Active)
        .unwrap();

    let restored_penalty = world
        .apply_identity_penalty(
            "agent-1",
            "evidence.fp.monitor",
            "possible false positive",
            10,
            10,
            "guardian-1",
            local_guardians(),
        )
        .unwrap();
    world
        .appeal_identity_penalty(restored_penalty, "agent-1", "counter evidence provided")
        .unwrap();
    world
        .resolve_identity_penalty_appeal(
            restored_penalty,
            "committee",
            true,
            "counter evidence accepted",
        )
        .unwrap();

    world
        .apply_identity_penalty(
            "agent-1",
            "evidence.sybil.open",
            "still under review",
            5,
            10,
            "guardian-2",
            local_guardians(),
        )
        .unwrap();

    let stats = world.governance_identity_penalty_monitor_stats(0);
    assert_eq!(stats.total_penalties, 2);
    assert_eq!(stats.appealed_penalties, 1);
    assert_eq!(stats.resolved_appeals, 1);
    assert_eq!(stats.appeal_accepted_penalties, 1);
    assert_eq!(stats.high_risk_open_penalties, 1);
    assert_eq!(stats.false_positive_rate_bps, 10_000);
}
