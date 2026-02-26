use super::super::*;
use serde_json::json;

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
