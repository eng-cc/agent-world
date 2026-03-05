use crate::runtime::state::ModuleReleaseRequestStatus;

#[test]
fn module_release_state_machine_runs_submit_shadow_approve_apply() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");
    register_agent(&mut world, "operator-1");

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
    assert_eq!(
        world.module_registry().active.get("m.loop.release"),
        Some(&"0.1.0".to_string())
    );
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

    assert_last_rejection_note(&world, action_id, "required roles are not fully approved");
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
