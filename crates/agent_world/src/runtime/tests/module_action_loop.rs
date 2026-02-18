use super::super::*;
use super::pos;

fn register_agent(world: &mut World, agent_id: &str) {
    world.submit_action(Action::RegisterAgent {
        agent_id: agent_id.to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");
}

fn base_manifest(module_id: &str, version: &str, wasm_hash: &str) -> ModuleManifest {
    ModuleManifest {
        module_id: module_id.to_string(),
        name: format!("module-{module_id}"),
        version: version.to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: wasm_hash.to_string(),
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits::default(),
    }
}

fn assert_last_rejection_note(world: &World, action_id: ActionId, expected: &str) {
    let event = world.journal().events.last().expect("last event");
    let WorldEventBody::Domain(DomainEvent::ActionRejected {
        action_id: rejected_action_id,
        reason: RejectReason::RuleDenied { notes },
    }) = &event.body
    else {
        panic!(
            "expected action rejected rule denied event: {:?}",
            event.body
        );
    };
    assert_eq!(*rejected_action_id, action_id);
    assert!(
        notes.iter().any(|note| note.contains(expected)),
        "missing expected note `{expected}` in {notes:?}"
    );
}

#[test]
fn deploy_module_artifact_action_registers_artifact_bytes() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");

    let wasm_bytes = b"module-action-loop-deploy".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes: wasm_bytes.clone(),
    });
    world.step().expect("deploy artifact");

    let event = world.journal().events.last().expect("last event");
    let WorldEventBody::Domain(DomainEvent::ModuleArtifactDeployed {
        publisher_agent_id,
        wasm_hash: event_hash,
        bytes_len,
    }) = &event.body
    else {
        panic!("expected module artifact deployed event: {:?}", event.body);
    };
    assert_eq!(publisher_agent_id, "publisher-1");
    assert_eq!(event_hash, &wasm_hash);
    assert_eq!(*bytes_len, wasm_bytes.len() as u64);

    let loaded = world.load_module(&wasm_hash).expect("load deployed module");
    assert_eq!(loaded.wasm_hash, wasm_hash);
    assert_eq!(loaded.bytes, wasm_bytes);
}

#[test]
fn deploy_module_artifact_action_rejects_hash_mismatch() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");

    let action_id = world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "publisher-1".to_string(),
        wasm_hash: "sha256-mismatch".to_string(),
        wasm_bytes: b"module-action-loop-deploy-mismatch".to_vec(),
    });
    world.step().expect("deploy mismatch action");

    assert_last_rejection_note(&world, action_id, "artifact hash mismatch");
}

#[test]
fn install_module_from_artifact_action_runs_governance_closure() {
    let mut world = World::new();
    register_agent(&mut world, "installer-1");

    let wasm_bytes = b"module-action-loop-install".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "installer-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    let manifest = base_manifest("m.loop.active", "0.1.0", &wasm_hash);
    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "installer-1".to_string(),
        manifest: manifest.clone(),
        activate: true,
    });
    world.step().expect("install module action");

    let event = world.journal().events.last().expect("last event");
    let WorldEventBody::Domain(DomainEvent::ModuleInstalled {
        installer_agent_id,
        module_id,
        module_version,
        active,
        proposal_id,
        manifest_hash,
    }) = &event.body
    else {
        panic!("expected module installed event: {:?}", event.body);
    };
    assert_eq!(installer_agent_id, "installer-1");
    assert_eq!(module_id, "m.loop.active");
    assert_eq!(module_version, "0.1.0");
    assert!(*active);
    assert!(!manifest_hash.is_empty());

    let key = ModuleRegistry::record_key(&manifest.module_id, &manifest.version);
    assert!(world.module_registry().records.contains_key(&key));
    assert_eq!(
        world.module_registry().active.get(&manifest.module_id),
        Some(&manifest.version)
    );
    assert!(matches!(
        world
            .proposals()
            .get(proposal_id)
            .map(|proposal| &proposal.status),
        Some(ProposalStatus::Applied { .. })
    ));
}

#[test]
fn install_module_from_artifact_action_without_activate_keeps_module_inactive() {
    let mut world = World::new();
    register_agent(&mut world, "installer-1");

    let wasm_bytes = b"module-action-loop-install-inactive".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "installer-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    let manifest = base_manifest("m.loop.inactive", "0.1.0", &wasm_hash);
    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "installer-1".to_string(),
        manifest: manifest.clone(),
        activate: false,
    });
    world.step().expect("install module inactive action");

    let event = world.journal().events.last().expect("last event");
    let WorldEventBody::Domain(DomainEvent::ModuleInstalled {
        active, module_id, ..
    }) = &event.body
    else {
        panic!("expected module installed event: {:?}", event.body);
    };
    assert!(!*active);
    assert_eq!(module_id, "m.loop.inactive");

    let key = ModuleRegistry::record_key(&manifest.module_id, &manifest.version);
    assert!(world.module_registry().records.contains_key(&key));
    assert!(!world
        .module_registry()
        .active
        .contains_key(&manifest.module_id));
}

#[test]
fn install_module_from_artifact_action_rejects_missing_artifact() {
    let mut world = World::new();
    register_agent(&mut world, "installer-1");

    let action_id = world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "installer-1".to_string(),
        manifest: base_manifest("m.loop.missing", "0.1.0", "sha256-missing"),
        activate: true,
    });
    world.step().expect("install missing artifact action");

    assert_last_rejection_note(&world, action_id, "module artifact missing");
    assert!(world.module_registry().records.is_empty());
    assert!(world.module_registry().active.is_empty());
}
