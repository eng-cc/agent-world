use crate::runtime::state::ModuleReleaseRequestStatus;

fn bind_release_roles(world: &mut World, operator_agent_id: &str, target_agent_id: &str, roles: &[&str]) {
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
    assert!(
        !world
            .state()
            .module_release_role_bindings
            .contains_key("auditor-1")
    );
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
