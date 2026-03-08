use crate::runtime::state::ModuleReleaseRequestStatus;

const LOCAL_FINALITY_SIGNER_1: &str = "governance.local.finality.signer.1";
const LOCAL_FINALITY_SIGNER_2: &str = "governance.local.finality.signer.2";

fn bind_release_roles(
    world: &mut World,
    operator_agent_id: &str,
    target_agent_id: &str,
    roles: &[&str],
) {
    world.submit_action(Action::ModuleReleaseBindRoles {
        operator_agent_id: operator_agent_id.to_string(),
        target_agent_id: target_agent_id.to_string(),
        roles: roles.iter().map(|role| role.to_string()).collect(),
    });
    world.step().expect("bind module release roles");
}

fn assert_rule_denied_note_for_action(world: &World, action_id: ActionId, expected: &str) {
    let notes = world
        .journal()
        .events
        .iter()
        .rev()
        .find_map(|event| match &event.body {
            WorldEventBody::Domain(DomainEvent::ActionRejected {
                action_id: rejected_action_id,
                reason: RejectReason::RuleDenied { notes },
            }) if *rejected_action_id == action_id => Some(notes.clone()),
            _ => None,
        })
        .expect("action rejected rule denied event");
    assert!(
        notes.iter().any(|note| note.contains(expected)),
        "missing expected note `{expected}` in {notes:?}"
    );
}

fn sample_profile_changes() -> ModuleProfileChanges {
    ModuleProfileChanges {
        product_profiles: vec![ProductProfileV1 {
            product_id: "module_rack".to_string(),
            role_tag: "scale".to_string(),
            maintenance_sink: vec![MaterialStack::new("hardware_part", 1)],
            tradable: true,
            unlock_stage: "scale_out".to_string(),
        }],
        recipe_profiles: vec![RecipeProfileV1 {
            recipe_id: "recipe.assembler.module_rack".to_string(),
            bottleneck_tags: vec!["control_chip".to_string()],
            stage_gate: "scale_out".to_string(),
            preferred_factory_tags: vec!["assembler".to_string()],
        }],
        factory_profiles: vec![FactoryProfileV1 {
            factory_id: "factory.assembler.mk1".to_string(),
            tier: 2,
            recipe_slots: 4,
            tags: vec!["assembler".to_string()],
        }],
    }
}

fn duplicate_profile_changes() -> ModuleProfileChanges {
    ModuleProfileChanges {
        product_profiles: vec![
            ProductProfileV1 {
                product_id: "dup_product".to_string(),
                role_tag: "scale".to_string(),
                maintenance_sink: Vec::new(),
                tradable: true,
                unlock_stage: "scale_out".to_string(),
            },
            ProductProfileV1 {
                product_id: "dup_product".to_string(),
                role_tag: "energy".to_string(),
                maintenance_sink: Vec::new(),
                tradable: true,
                unlock_stage: "scale_out".to_string(),
            },
        ],
        recipe_profiles: Vec::new(),
        factory_profiles: Vec::new(),
    }
}

fn duplicate_factory_profile_changes() -> ModuleProfileChanges {
    ModuleProfileChanges {
        product_profiles: Vec::new(),
        recipe_profiles: Vec::new(),
        factory_profiles: vec![
            FactoryProfileV1 {
                factory_id: "dup_factory".to_string(),
                tier: 1,
                recipe_slots: 2,
                tags: vec!["assembly".to_string()],
            },
            FactoryProfileV1 {
                factory_id: "dup_factory".to_string(),
                tier: 2,
                recipe_slots: 3,
                tags: vec!["assembly".to_string()],
            },
        ],
    }
}

fn duplicate_recipe_profile_changes() -> ModuleProfileChanges {
    ModuleProfileChanges {
        product_profiles: Vec::new(),
        recipe_profiles: vec![
            RecipeProfileV1 {
                recipe_id: "dup_recipe".to_string(),
                bottleneck_tags: vec!["control_chip".to_string()],
                stage_gate: "scale_out".to_string(),
                preferred_factory_tags: vec!["assembler".to_string()],
            },
            RecipeProfileV1 {
                recipe_id: "dup_recipe".to_string(),
                bottleneck_tags: vec!["maintenance".to_string()],
                stage_gate: "scale_out".to_string(),
                preferred_factory_tags: vec!["assembler".to_string()],
            },
        ],
        factory_profiles: Vec::new(),
    }
}

fn bind_attestor_node_identity(world: &mut World, node_id: &str) {
    let public_key_hex = util::sha256_hex(node_id.as_bytes());
    world
        .bind_node_identity(node_id, public_key_hex.as_str())
        .expect("bind attestor node identity");
}

fn set_module_release_attestation_epoch_snapshot(
    world: &mut World,
    threshold: u16,
    signer_node_ids: &[&str],
) {
    let epoch_len = world
        .governance_execution_policy()
        .epoch_length_ticks
        .max(1);
    let epoch_id = world.state().time / epoch_len;
    world
        .set_governance_finality_epoch_snapshot(GovernanceFinalityEpochSnapshot {
            epoch_id,
            threshold,
            signer_node_ids: signer_node_ids
                .iter()
                .map(|signer| signer.to_string())
                .collect(),
            ..GovernanceFinalityEpochSnapshot::default()
        })
        .expect("set module release attestation epoch snapshot");
}

fn prepare_module_release_apply_ready_request(
    world: &mut World,
    requester_agent_id: &str,
    operator_agent_id: &str,
    module_id: &str,
) -> u64 {
    let wasm_bytes = format!("module-release-ready-{module_id}").into_bytes();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: requester_agent_id.to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: requester_agent_id.to_string(),
        manifest: base_manifest(module_id, "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: operator_agent_id.to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(world, operator_agent_id, operator_agent_id, &["security"]);
    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: operator_agent_id.to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve module release role");
    set_module_release_attestation_epoch_snapshot(
        world,
        2,
        &[LOCAL_FINALITY_SIGNER_1, LOCAL_FINALITY_SIGNER_2],
    );
    for (index, signer_node_id) in [LOCAL_FINALITY_SIGNER_1, LOCAL_FINALITY_SIGNER_2]
        .iter()
        .enumerate()
    {
        world.submit_action(Action::ModuleReleaseSubmitAttestation {
            operator_agent_id: operator_agent_id.to_string(),
            request_id,
            signer_node_id: signer_node_id.to_string(),
            platform: "linux-x86_64".to_string(),
            build_manifest_hash: util::sha256_hex(
                format!("release-ready-build-{request_id}-{index}").as_bytes(),
            ),
            source_hash: util::sha256_hex(
                format!("release-ready-source-{request_id}-{index}").as_bytes(),
            ),
            wasm_hash: wasm_hash.clone(),
            proof_cid: format!("bafyreadyapply{request_id}{index:02}"),
        });
        world
            .step()
            .expect("submit module release attestation before apply");
    }
    request_id
}

#[test]
fn module_release_state_machine_runs_submit_shadow_approve_apply() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    set_module_release_attestation_epoch_snapshot(
        &mut world,
        2,
        &[LOCAL_FINALITY_SIGNER_1, LOCAL_FINALITY_SIGNER_2],
    );

    let wasm_bytes = b"module-release-state-machine".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    let manifest = base_manifest("m.loop.release", "0.1.0", &wasm_hash);
    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: manifest.clone(),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["runtime".to_string(), "security".to_string()],
        profile_changes: sample_profile_changes(),
    });
    world.step().expect("submit module release request");

    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested {
            request_id,
            requester_agent_id,
            required_roles,
            ..
        }) => {
            assert_eq!(requester_agent_id, "publisher-1");
            assert_eq!(
                required_roles,
                &vec!["runtime".to_string(), "security".to_string()]
            );
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };
    assert!(request_id > 0);
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::Requested)
    ));
    let mapping = world
        .state()
        .module_release_manifest_mappings
        .get(&request_id)
        .expect("module release manifest mapping state");
    assert_eq!(mapping.status, ModuleReleaseRequestStatus::Requested);
    assert_eq!(mapping.module_id, "m.loop.release");
    assert_eq!(mapping.release_id, format!("release-{request_id}"));

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    let shadow_manifest_hash = match &world.journal().events.last().expect("shadow event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseShadowed {
            request_id: event_request_id,
            operator_agent_id,
            manifest_hash,
        }) => {
            assert_eq!(*event_request_id, request_id);
            assert_eq!(operator_agent_id, "operator-1");
            manifest_hash.clone()
        }
        other => panic!("expected module release shadowed event: {other:?}"),
    };
    assert!(!shadow_manifest_hash.is_empty());
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::Shadowed)
    ));
    let mapping = world
        .state()
        .module_release_manifest_mappings
        .get(&request_id)
        .expect("module release manifest mapping after shadow");
    assert_eq!(mapping.status, ModuleReleaseRequestStatus::Shadowed);
    assert_eq!(
        mapping.shadow_manifest_hash.as_deref(),
        Some(shadow_manifest_hash.as_str())
    );
    bind_release_roles(&mut world, "operator-1", "operator-1", &["security"]);
    bind_release_roles(&mut world, "operator-1", "publisher-1", &["runtime"]);

    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve module release security role");
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::PartiallyApproved)
    ));

    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "publisher-1".to_string(),
        request_id,
        role: "runtime".to_string(),
    });
    world.step().expect("approve module release runtime role");
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::Approved)
    ));
    let build_manifest_hash = util::sha256_hex(b"state-machine-build-manifest");
    let source_hash = util::sha256_hex(b"state-machine-source-hash");
    world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: LOCAL_FINALITY_SIGNER_1.to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash: build_manifest_hash.clone(),
        source_hash: source_hash.clone(),
        wasm_hash: wasm_hash.clone(),
        proof_cid: "bafyreleaseattestsm001".to_string(),
    });
    world.step().expect("submit attestation signer1");
    world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: LOCAL_FINALITY_SIGNER_2.to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash,
        source_hash,
        wasm_hash: wasm_hash.clone(),
        proof_cid: "bafyreleaseattestsm002".to_string(),
    });
    world.step().expect("submit attestation signer2");
    assert_eq!(
        world
            .state()
            .module_release_manifest_mappings
            .get(&request_id)
            .expect("module release manifest mapping state")
            .attestation_count,
        2
    );

    world.submit_action(Action::ModuleReleaseApply {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("apply module release request");

    let apply_event = world
        .journal()
        .events
        .iter()
        .rev()
        .find_map(|event| match &event.body {
            WorldEventBody::Domain(DomainEvent::ModuleReleaseApplied {
                request_id: event_request_id,
                operator_agent_id,
                installer_agent_id,
                module_id,
                module_version,
                proposal_id,
                manifest_hash,
                ..
            }) if *event_request_id == request_id => Some((
                operator_agent_id.clone(),
                installer_agent_id.clone(),
                module_id.clone(),
                module_version.clone(),
                *proposal_id,
                manifest_hash.clone(),
            )),
            _ => None,
        })
        .expect("module release applied event");
    let (
        operator_agent_id,
        installer_agent_id,
        module_id,
        module_version,
        proposal_id,
        applied_manifest_hash,
    ) = apply_event;
    assert_eq!(operator_agent_id, "operator-1");
    assert_eq!(installer_agent_id, "publisher-1");
    assert_eq!(module_id, "m.loop.release");
    assert_eq!(module_version, "0.1.0");
    assert!(proposal_id > 0);
    assert!(!applied_manifest_hash.is_empty());

    let release_state = world
        .state()
        .module_release_requests
        .get(&request_id)
        .expect("module release request state");
    assert_eq!(release_state.status, ModuleReleaseRequestStatus::Applied);
    assert_eq!(
        release_state.applied_manifest_hash.as_deref(),
        Some(applied_manifest_hash.as_str())
    );
    assert_eq!(release_state.applied_proposal_id, Some(proposal_id));
    let mapping = world
        .state()
        .module_release_manifest_mappings
        .get(&request_id)
        .expect("module release manifest mapping state after apply");
    assert_eq!(mapping.status, ModuleReleaseRequestStatus::Applied);
    assert_eq!(
        mapping.applied_manifest_hash.as_deref(),
        Some(applied_manifest_hash.as_str())
    );
    assert_eq!(mapping.applied_proposal_id, Some(proposal_id));
    assert_eq!(
        world.module_registry().active.get("m.loop.release"),
        Some(&"0.1.0".to_string())
    );

    let product = world
        .product_profile("module_rack")
        .expect("product profile applied");
    assert_eq!(product.role_tag, "scale");
    let recipe = world
        .recipe_profile("recipe.assembler.module_rack")
        .expect("recipe profile applied");
    assert_eq!(recipe.stage_gate, "scale_out");
    let factory = world
        .factory_profile("factory.assembler.mk1")
        .expect("factory profile applied");
    assert_eq!(factory.recipe_slots, 4);

    let snapshot = world.snapshot();
    let restored =
        World::from_snapshot(snapshot, world.journal().clone()).expect("restore from snapshot");
    assert!(restored.product_profile("module_rack").is_some());
    assert!(restored
        .recipe_profile("recipe.assembler.module_rack")
        .is_some());
    assert!(restored.factory_profile("factory.assembler.mk1").is_some());
}

#[test]
fn module_release_submit_attestation_persists_audit_evidence() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    bind_attestor_node_identity(&mut world, "attestor-node-1");

    let wasm_bytes = b"module-release-attestation-audit".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.attest", "0.1.0", &wasm_hash),
        activate: false,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["runtime".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let build_manifest_hash = util::sha256_hex(b"attest-build-manifest");
    let source_hash = util::sha256_hex(b"attest-source-hash");
    world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: "attestor-node-1".to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash: build_manifest_hash.clone(),
        source_hash: source_hash.clone(),
        wasm_hash: wasm_hash.clone(),
        proof_cid: "bafyreleaseattest0001".to_string(),
    });
    world.step().expect("submit module release attestation");

    match &world
        .journal()
        .events
        .last()
        .expect("attestation event")
        .body
    {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseAttested {
            request_id: event_request_id,
            operator_agent_id,
            signer_node_id,
            platform,
            build_manifest_hash: event_build_manifest_hash,
            source_hash: event_source_hash,
            wasm_hash: event_wasm_hash,
            proof_cid,
        }) => {
            assert_eq!(*event_request_id, request_id);
            assert_eq!(operator_agent_id, "operator-1");
            assert_eq!(signer_node_id, "attestor-node-1");
            assert_eq!(platform, "linux-x86_64");
            assert_eq!(event_build_manifest_hash, &build_manifest_hash);
            assert_eq!(event_source_hash, &source_hash);
            assert_eq!(event_wasm_hash, &wasm_hash);
            assert_eq!(proof_cid, "bafyreleaseattest0001");
        }
        other => panic!("expected module release attested event: {other:?}"),
    }

    let request = world
        .state()
        .module_release_requests
        .get(&request_id)
        .expect("module release request state");
    let attestation = request
        .attestations
        .get("attestor-node-1|linux-x86_64")
        .expect("attestation state");
    assert_eq!(attestation.proof_cid, "bafyreleaseattest0001");
    assert_eq!(attestation.wasm_hash, wasm_hash);
    let mapping = world
        .state()
        .module_release_manifest_mappings
        .get(&request_id)
        .expect("module release mapping state");
    assert_eq!(mapping.attestation_count, 1);
}

#[test]
fn module_release_submit_attestation_rejects_conflicting_duplicate() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    bind_attestor_node_identity(&mut world, "attestor-node-1");

    let wasm_bytes = b"module-release-attestation-conflict".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.attest.dup", "0.1.0", &wasm_hash),
        activate: false,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["runtime".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let build_manifest_hash = util::sha256_hex(b"attest-dup-build-manifest");
    let source_hash = util::sha256_hex(b"attest-dup-source-hash");
    world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: "attestor-node-1".to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash: build_manifest_hash.clone(),
        source_hash: source_hash.clone(),
        wasm_hash: wasm_hash.clone(),
        proof_cid: "bafyreleaseattestdup0001".to_string(),
    });
    world.step().expect("submit first attestation");

    let action_id = world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: "attestor-node-1".to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash,
        source_hash,
        wasm_hash,
        proof_cid: "bafyreleaseattestdup0002".to_string(),
    });
    world.step().expect("submit conflicting attestation");

    assert_rule_denied_note_for_action(&world, action_id, "conflicting attestation already exists");
    let request = world
        .state()
        .module_release_requests
        .get(&request_id)
        .expect("module release request state");
    assert_eq!(request.attestations.len(), 1);
    let mapping = world
        .state()
        .module_release_manifest_mappings
        .get(&request_id)
        .expect("module release mapping state");
    assert_eq!(mapping.attestation_count, 1);
}

#[test]
fn module_release_apply_rejects_when_attestation_threshold_not_met() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    set_module_release_attestation_epoch_snapshot(
        &mut world,
        2,
        &[LOCAL_FINALITY_SIGNER_1, LOCAL_FINALITY_SIGNER_2],
    );

    let wasm_bytes = b"module-release-threshold-not-met".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.threshold", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(&mut world, "operator-1", "operator-1", &["security"]);
    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve required role");
    world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: LOCAL_FINALITY_SIGNER_1.to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash: util::sha256_hex(b"threshold-build-manifest"),
        source_hash: util::sha256_hex(b"threshold-source-hash"),
        wasm_hash,
        proof_cid: "bafyreleaseattestthreshold001".to_string(),
    });
    world.step().expect("submit single attestation");

    let action_id = world.submit_action(Action::ModuleReleaseApply {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("apply module release request");

    assert_rule_denied_note_for_action(&world, action_id, "attestation threshold not met");
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::Approved)
    ));
}

#[test]
fn module_release_apply_rejects_when_attestor_not_in_epoch_snapshot() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    set_module_release_attestation_epoch_snapshot(&mut world, 1, &[LOCAL_FINALITY_SIGNER_1]);

    let wasm_bytes = b"module-release-attestor-outside-snapshot".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.snapshot-filter", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(&mut world, "operator-1", "operator-1", &["security"]);
    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve required role");
    world.submit_action(Action::ModuleReleaseSubmitAttestation {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        signer_node_id: LOCAL_FINALITY_SIGNER_2.to_string(),
        platform: "linux-x86_64".to_string(),
        build_manifest_hash: util::sha256_hex(b"snapshot-filter-build-manifest"),
        source_hash: util::sha256_hex(b"snapshot-filter-source-hash"),
        wasm_hash,
        proof_cid: "bafyreleaseattestsnapshot001".to_string(),
    });
    world.step().expect("submit out-of-snapshot attestation");

    let action_id = world.submit_action(Action::ModuleReleaseApply {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("apply module release request");

    assert_rule_denied_note_for_action(&world, action_id, "attestation threshold not met");
}

#[test]
fn module_release_shadow_rejects_duplicate_profile_changes() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-dup-profile".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.dup-profile", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: duplicate_profile_changes(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let action_id = world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("shadow module release request with dup profiles");
    assert_rule_denied_note_for_action(&world, action_id, "duplicate product profile_id");
}

#[test]
fn module_release_shadow_rejects_duplicate_factory_profile_changes() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-dup-factory-profile".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.dup-factory", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: duplicate_factory_profile_changes(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let action_id = world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("shadow module release request with dup factory profiles");
    assert_rule_denied_note_for_action(&world, action_id, "duplicate factory profile_id");
}

#[test]
fn module_release_shadow_rejects_duplicate_recipe_profile_changes() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-dup-recipe-profile".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.dup-recipe", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: duplicate_recipe_profile_changes(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let action_id = world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("shadow module release request with dup recipe profiles");
    assert_rule_denied_note_for_action(&world, action_id, "duplicate recipe profile_id");
}

#[test]
fn module_release_shadow_rejects_missing_artifact_identity() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-missing-identity".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    let mut manifest = base_manifest("m.loop.release.missing-identity", "0.1.0", &wasm_hash);
    manifest.artifact_identity = None;
    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest,
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let action_id = world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("shadow module release request missing identity");
    assert_rule_denied_note_for_action(&world, action_id, "artifact_identity is required");
}

#[test]
fn module_release_shadow_rejects_unsigned_artifact_identity_signature() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-unsigned-identity".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    let mut manifest = base_manifest("m.loop.release.unsigned-identity", "0.1.0", &wasm_hash);
    let mut identity = manifest
        .artifact_identity
        .clone()
        .expect("base manifest identity");
    identity.artifact_signature = "unsigned:tampered".to_string();
    manifest.artifact_identity = Some(identity);

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest,
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    let action_id = world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("shadow module release request unsigned identity");
    assert_rule_denied_note_for_action(&world, action_id, "unsigned signature is forbidden");
}

#[test]
fn module_release_apply_rejects_when_required_roles_are_missing() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-missing-roles".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.missing-role", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string(), "runtime".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(&mut world, "operator-1", "operator-1", &["security"]);

    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve one required role");
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::PartiallyApproved)
    ));

    let action_id = world.submit_action(Action::ModuleReleaseApply {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("apply module release request with missing roles");

    assert_rule_denied_note_for_action(&world, action_id, "required roles are not fully approved");
    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::PartiallyApproved)
    ));
}

#[test]
fn module_release_duplicate_role_approval_is_idempotent_for_same_approver() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");

    let wasm_bytes = b"module-release-duplicate-role-approval".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.dup-role", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "publisher-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(&mut world, "publisher-1", "publisher-1", &["security"]);

    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "publisher-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve role first time");

    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "publisher-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve role second time");

    let request = world
        .state()
        .module_release_requests
        .get(&request_id)
        .expect("module release request state");
    assert_eq!(request.status, ModuleReleaseRequestStatus::Approved);
    assert_eq!(request.role_approvals.len(), 1);
    assert_eq!(
        request.role_approvals.get("security"),
        Some(&"publisher-1".to_string())
    );
}

#[test]
fn module_release_approve_role_rejects_when_role_not_required() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-role-not-required".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.role-not-required", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(&mut world, "operator-1", "operator-1", &["runtime"]);

    let action_id = world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "runtime".to_string(),
    });
    world.step().expect("reject role not required");
    assert_rule_denied_note_for_action(&world, action_id, "role not required");
}

#[test]
fn module_release_approve_role_rejects_when_role_already_approved_by_other() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    register_agent(&mut world, "operator-2");

    let wasm_bytes = b"module-release-role-already-approved".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.role-already-approved", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");
    bind_release_roles(&mut world, "operator-1", "operator-1", &["security"]);
    bind_release_roles(&mut world, "operator-1", "operator-2", &["security"]);

    world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("approve required role");

    let action_id = world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-2".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world.step().expect("reject role already approved");
    assert_rule_denied_note_for_action(&world, action_id, "already approved");
}

#[test]
fn module_release_reject_moves_request_to_rejected() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-reject".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.rejected", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseReject {
        rejector_agent_id: "operator-1".to_string(),
        request_id,
        reason: "policy violation".to_string(),
    });
    world.step().expect("reject module release request");
    assert!(matches!(
        world.journal().events.last().map(|event| &event.body),
        Some(WorldEventBody::Domain(DomainEvent::ModuleReleaseRejected {
            request_id: event_request_id,
            rejector_agent_id,
            reason,
        })) if *event_request_id == request_id
            && rejector_agent_id == "operator-1"
            && reason == "policy violation"
    ));

    let request = world
        .state()
        .module_release_requests
        .get(&request_id)
        .expect("module release request state");
    assert_eq!(request.status, ModuleReleaseRequestStatus::Rejected);
    assert_eq!(request.rejected_reason.as_deref(), Some("policy violation"));
}

#[test]
fn module_release_approve_role_rejects_when_approver_role_binding_is_missing() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

    let wasm_bytes = b"module-release-unbound-approver".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy module artifact");

    world.submit_action(Action::ModuleReleaseSubmit {
        requester_agent_id: "publisher-1".to_string(),
        manifest: base_manifest("m.loop.release.unbound", "0.1.0", &wasm_hash),
        activate: true,
        install_target: ModuleInstallTarget::SelfAgent,
        required_roles: vec!["security".to_string()],
        profile_changes: ModuleProfileChanges::default(),
    });
    world.step().expect("submit module release request");
    let request_id = match &world.journal().events.last().expect("submit event").body {
        WorldEventBody::Domain(DomainEvent::ModuleReleaseRequested { request_id, .. }) => {
            *request_id
        }
        other => panic!("expected module release requested event: {other:?}"),
    };

    world.submit_action(Action::ModuleReleaseShadow {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world.step().expect("shadow module release request");

    let action_id = world.submit_action(Action::ModuleReleaseApproveRole {
        approver_agent_id: "operator-1".to_string(),
        request_id,
        role: "security".to_string(),
    });
    world
        .step()
        .expect("reject approve role without role binding");
    assert_rule_denied_note_for_action(&world, action_id, "approver role binding missing");
}

#[test]
fn module_release_bind_roles_normalizes_and_updates_state() {
    let mut world = World::new();
    register_agent(&mut world, "operator-1");
    register_agent(&mut world, "auditor-1");

    world.submit_action(Action::ModuleReleaseBindRoles {
        operator_agent_id: "operator-1".to_string(),
        target_agent_id: "auditor-1".to_string(),
        roles: vec![
            "Security".to_string(),
            " runtime ".to_string(),
            "security".to_string(),
            "".to_string(),
        ],
    });
    world.step().expect("bind module release roles");
    assert!(matches!(
        world.journal().events.last().map(|event| &event.body),
        Some(WorldEventBody::Domain(DomainEvent::ModuleReleaseRolesBound {
            operator_agent_id,
            target_agent_id,
            roles,
        })) if operator_agent_id == "operator-1"
            && target_agent_id == "auditor-1"
            && roles == &vec!["runtime".to_string(), "security".to_string()]
    ));
    let bound_roles = world
        .state()
        .module_release_role_bindings
        .get("auditor-1")
        .expect("bound roles");
    assert!(bound_roles.contains("security"));
    assert!(bound_roles.contains("runtime"));

    world.submit_action(Action::ModuleReleaseBindRoles {
        operator_agent_id: "operator-1".to_string(),
        target_agent_id: "auditor-1".to_string(),
        roles: Vec::new(),
    });
    world.step().expect("unbind module release roles");
    assert!(!world
        .state()
        .module_release_role_bindings
        .contains_key("auditor-1"));
}

#[test]
fn rollback_module_instance_reverts_to_historical_version_and_emits_audit() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_v1_bytes = b"module-rollback-v1".to_vec();
    let wasm_v1_hash = util::sha256_hex(&wasm_v1_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v1_hash.clone(),
        wasm_bytes: wasm_v1_bytes,
    });
    world.step().expect("deploy v1 artifact");

    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.rollback", "0.1.0", &wasm_v1_hash),
        activate: true,
    });
    world.step().expect("install v1");
    let instance_id = match &world.journal().events.last().expect("install event").body {
        WorldEventBody::Domain(DomainEvent::ModuleInstalled { instance_id, .. }) => {
            instance_id.clone()
        }
        other => panic!("expected module installed event: {other:?}"),
    };

    let wasm_v2_bytes = b"module-rollback-v2".to_vec();
    let wasm_v2_hash = util::sha256_hex(&wasm_v2_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v2_hash.clone(),
        wasm_bytes: wasm_v2_bytes,
    });
    world.step().expect("deploy v2 artifact");

    world.submit_action(Action::UpgradeModuleFromArtifact {
        upgrader_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        from_module_version: "0.1.0".to_string(),
        manifest: base_manifest("m.loop.rollback", "0.2.0", &wasm_v2_hash),
        activate: true,
    });
    world.step().expect("upgrade to v2");

    world.submit_action(Action::RollbackModuleInstance {
        operator_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        target_module_version: "0.1.0".to_string(),
    });
    world.step().expect("rollback to v1");

    let rollback_event = world
        .journal()
        .events
        .iter()
        .rev()
        .find_map(|event| match &event.body {
            WorldEventBody::Domain(DomainEvent::ModuleRollbackApplied {
                instance_id: event_instance_id,
                from_module_version,
                to_module_version,
                wasm_hash,
                proposal_id,
                ..
            }) if event_instance_id == &instance_id => Some((
                from_module_version.clone(),
                to_module_version.clone(),
                wasm_hash.clone(),
                *proposal_id,
            )),
            _ => None,
        })
        .expect("module rollback event");
    assert_eq!(rollback_event.0, "0.2.0");
    assert_eq!(rollback_event.1, "0.1.0");
    assert_eq!(rollback_event.2, wasm_v1_hash);
    assert!(rollback_event.3 > 0);

    let instance = world
        .state()
        .module_instances
        .get(&instance_id)
        .expect("instance state");
    assert_eq!(instance.module_version, "0.1.0");
    assert_eq!(instance.wasm_hash, rollback_event.2);
}

#[test]
fn rollback_module_instance_rejects_when_target_version_missing() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_bytes = b"module-rollback-missing-target".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.rollback.missing", "0.1.0", &wasm_hash),
        activate: true,
    });
    world.step().expect("install module");
    let instance_id = match &world.journal().events.last().expect("install event").body {
        WorldEventBody::Domain(DomainEvent::ModuleInstalled { instance_id, .. }) => {
            instance_id.clone()
        }
        other => panic!("expected module installed event: {other:?}"),
    };

    let action_id = world.submit_action(Action::RollbackModuleInstance {
        operator_agent_id: "owner-1".to_string(),
        instance_id,
        target_module_version: "9.9.9".to_string(),
    });
    world.step().expect("reject missing rollback target");
    assert_rule_denied_note_for_action(&world, action_id, "target version not found");
}

#[test]
fn rollback_module_instance_rejects_when_operator_does_not_own_instance() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");
    register_agent(&mut world, "owner-2");

    let wasm_bytes = b"module-rollback-owner-mismatch".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.rollback.owner", "0.1.0", &wasm_hash),
        activate: true,
    });
    world.step().expect("install module");
    let instance_id = match &world.journal().events.last().expect("install event").body {
        WorldEventBody::Domain(DomainEvent::ModuleInstalled { instance_id, .. }) => {
            instance_id.clone()
        }
        other => panic!("expected module installed event: {other:?}"),
    };

    let action_id = world.submit_action(Action::RollbackModuleInstance {
        operator_agent_id: "owner-2".to_string(),
        instance_id,
        target_module_version: "0.1.0".to_string(),
    });
    world.step().expect("reject owner mismatch rollback");
    assert_rule_denied_note_for_action(&world, action_id, "does not own instance");
}

#[test]
fn rollback_module_instance_rejects_when_target_interface_is_incompatible() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_v1_bytes = b"module-rollback-incompatible-v1".to_vec();
    let wasm_v1_hash = util::sha256_hex(&wasm_v1_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v1_hash.clone(),
        wasm_bytes: wasm_v1_bytes,
    });
    world.step().expect("deploy v1 artifact");

    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.rollback.incompatible", "0.1.0", &wasm_v1_hash),
        activate: true,
    });
    world.step().expect("install v1");
    let instance_id = match &world.journal().events.last().expect("install event").body {
        WorldEventBody::Domain(DomainEvent::ModuleInstalled { instance_id, .. }) => {
            instance_id.clone()
        }
        other => panic!("expected module installed event: {other:?}"),
    };

    let wasm_v2_bytes = b"module-rollback-incompatible-v2".to_vec();
    let wasm_v2_hash = util::sha256_hex(&wasm_v2_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v2_hash.clone(),
        wasm_bytes: wasm_v2_bytes,
    });
    world.step().expect("deploy v2 artifact");

    let mut manifest_v2 = base_manifest("m.loop.rollback.incompatible", "0.2.0", &wasm_v2_hash);
    manifest_v2.exports.push("audit".to_string());
    world.submit_action(Action::UpgradeModuleFromArtifact {
        upgrader_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        from_module_version: "0.1.0".to_string(),
        manifest: manifest_v2,
        activate: true,
    });
    world.step().expect("upgrade to v2");

    let action_id = world.submit_action(Action::RollbackModuleInstance {
        operator_agent_id: "owner-1".to_string(),
        instance_id,
        target_module_version: "0.1.0".to_string(),
    });
    world
        .step()
        .expect("reject incompatible rollback target interface");
    assert_rule_denied_note_for_action(&world, action_id, "exports incompatible");
}

#[test]
fn install_module_rejects_without_finality_in_production_policy() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_bytes = b"module-install-prod-no-finality".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.enable_production_release_policy();
    let action_id = world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.prod.install.reject", "0.1.0", &wasm_hash),
        activate: true,
    });
    world.step().expect("reject install without finality");
    assert_rule_denied_note_for_action(&world, action_id, "local finality path is disabled");
}

#[test]
fn upgrade_and_rollback_reject_without_finality_in_production_policy() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_v1_bytes = b"module-upgrade-rollback-prod-no-finality-v1".to_vec();
    let wasm_v1_hash = util::sha256_hex(&wasm_v1_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v1_hash.clone(),
        wasm_bytes: wasm_v1_bytes,
    });
    world.step().expect("deploy v1 artifact");
    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.prod.upgrade.reject", "0.1.0", &wasm_v1_hash),
        activate: true,
    });
    world.step().expect("install v1");
    let instance_id = match &world.journal().events.last().expect("install event").body {
        WorldEventBody::Domain(DomainEvent::ModuleInstalled { instance_id, .. }) => {
            instance_id.clone()
        }
        other => panic!("expected module installed event: {other:?}"),
    };

    let wasm_v2_bytes = b"module-upgrade-rollback-prod-no-finality-v2".to_vec();
    let wasm_v2_hash = util::sha256_hex(&wasm_v2_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v2_hash.clone(),
        wasm_bytes: wasm_v2_bytes,
    });
    world.step().expect("deploy v2 artifact");
    world.submit_action(Action::UpgradeModuleFromArtifact {
        upgrader_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        from_module_version: "0.1.0".to_string(),
        manifest: base_manifest("m.loop.prod.upgrade.reject", "0.2.0", &wasm_v2_hash),
        activate: true,
    });
    world.step().expect("upgrade to v2");

    let wasm_v3_bytes = b"module-upgrade-rollback-prod-no-finality-v3".to_vec();
    let wasm_v3_hash = util::sha256_hex(&wasm_v3_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v3_hash.clone(),
        wasm_bytes: wasm_v3_bytes,
    });
    world.step().expect("deploy v3 artifact");

    world.enable_production_release_policy();
    let upgrade_action_id = world.submit_action(Action::UpgradeModuleFromArtifact {
        upgrader_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        from_module_version: "0.2.0".to_string(),
        manifest: base_manifest("m.loop.prod.upgrade.reject", "0.3.0", &wasm_v3_hash),
        activate: true,
    });
    world.step().expect("reject upgrade without finality");
    assert_rule_denied_note_for_action(
        &world,
        upgrade_action_id,
        "local finality path is disabled",
    );

    let rollback_action_id = world.submit_action(Action::RollbackModuleInstance {
        operator_agent_id: "owner-1".to_string(),
        instance_id,
        target_module_version: "0.1.0".to_string(),
    });
    world.step().expect("reject rollback without finality");
    assert_rule_denied_note_for_action(
        &world,
        rollback_action_id,
        "local finality path is disabled",
    );
}

#[test]
fn install_upgrade_rollback_with_finality_succeed_in_production_policy() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");
    set_test_governance_finality_epoch_snapshot(
        &mut world,
        2,
        &[TEST_FINALITY_SIGNER_NODE_1, TEST_FINALITY_SIGNER_NODE_2],
    );

    let wasm_v1_bytes = b"module-with-finality-prod-v1".to_vec();
    let wasm_v1_hash = util::sha256_hex(&wasm_v1_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v1_hash.clone(),
        wasm_bytes: wasm_v1_bytes,
    });
    world.step().expect("deploy v1 artifact");

    world.enable_production_release_policy();
    let install_manifest = base_manifest("m.loop.prod.with-finality", "0.1.0", &wasm_v1_hash);
    let install_finality = derive_module_action_finality_certificate(&world, |simulated| {
        simulated.submit_action(Action::InstallModuleFromArtifact {
            installer_agent_id: "owner-1".to_string(),
            manifest: install_manifest.clone(),
            activate: true,
        });
    });
    world.submit_action(Action::InstallModuleFromArtifactWithFinality {
        installer_agent_id: "owner-1".to_string(),
        manifest: install_manifest,
        activate: true,
        finality_certificate: install_finality,
    });
    world.step().expect("install with finality");
    let instance_id = match &world.journal().events.last().expect("install event").body {
        WorldEventBody::Domain(DomainEvent::ModuleInstalled { instance_id, .. }) => {
            instance_id.clone()
        }
        other => panic!("expected module installed event: {other:?}"),
    };

    let wasm_v2_bytes = b"module-with-finality-prod-v2".to_vec();
    let wasm_v2_hash = util::sha256_hex(&wasm_v2_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_v2_hash.clone(),
        wasm_bytes: wasm_v2_bytes,
    });
    world.step().expect("deploy v2 artifact");
    let upgrade_manifest = base_manifest("m.loop.prod.with-finality", "0.2.0", &wasm_v2_hash);
    let upgrade_finality = derive_module_action_finality_certificate(&world, |simulated| {
        simulated.submit_action(Action::UpgradeModuleFromArtifact {
            upgrader_agent_id: "owner-1".to_string(),
            instance_id: instance_id.clone(),
            from_module_version: "0.1.0".to_string(),
            manifest: upgrade_manifest.clone(),
            activate: true,
        });
    });
    world.submit_action(Action::UpgradeModuleFromArtifactWithFinality {
        upgrader_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        from_module_version: "0.1.0".to_string(),
        manifest: upgrade_manifest,
        activate: true,
        finality_certificate: upgrade_finality,
    });
    world.step().expect("upgrade with finality");

    let rollback_finality = derive_module_action_finality_certificate(&world, |simulated| {
        simulated.submit_action(Action::RollbackModuleInstance {
            operator_agent_id: "owner-1".to_string(),
            instance_id: instance_id.clone(),
            target_module_version: "0.1.0".to_string(),
        });
    });
    world.submit_action(Action::RollbackModuleInstanceWithFinality {
        operator_agent_id: "owner-1".to_string(),
        instance_id: instance_id.clone(),
        target_module_version: "0.1.0".to_string(),
        finality_certificate: rollback_finality,
    });
    world.step().expect("rollback with finality");

    let instance = world
        .state()
        .module_instances
        .get(&instance_id)
        .expect("module instance state");
    assert_eq!(instance.module_version, "0.1.0");
}

#[test]
fn module_release_apply_rejects_without_finality_in_production_policy() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    let request_id = prepare_module_release_apply_ready_request(
        &mut world,
        "publisher-1",
        "operator-1",
        "m.loop.release.prod.reject",
    );

    world.enable_production_release_policy();
    let action_id = world.submit_action(Action::ModuleReleaseApply {
        operator_agent_id: "operator-1".to_string(),
        request_id,
    });
    world
        .step()
        .expect("reject module release apply without finality");
    assert_rule_denied_note_for_action(&world, action_id, "local finality path is disabled");
}

#[test]
fn module_release_apply_with_finality_succeeds_in_production_policy() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");
    let request_id = prepare_module_release_apply_ready_request(
        &mut world,
        "publisher-1",
        "operator-1",
        "m.loop.release.prod.with-finality",
    );

    world.enable_production_release_policy();
    let finality_certificate = derive_module_action_finality_certificate(&world, |simulated| {
        simulated.submit_action(Action::ModuleReleaseApply {
            operator_agent_id: "operator-1".to_string(),
            request_id,
        });
    });
    world.submit_action(Action::ModuleReleaseApplyWithFinality {
        operator_agent_id: "operator-1".to_string(),
        request_id,
        finality_certificate,
    });
    world
        .step()
        .expect("apply module release with finality in production");

    assert!(matches!(
        world
            .state()
            .module_release_requests
            .get(&request_id)
            .map(|item| item.status),
        Some(ModuleReleaseRequestStatus::Applied)
    ));
}
