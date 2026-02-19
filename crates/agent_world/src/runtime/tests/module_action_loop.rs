use super::super::*;
use super::pos;
use crate::simulator::{ModuleInstallTarget, ResourceKind};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const SOURCE_COMPILER_ENV: &str = "AGENT_WORLD_MODULE_SOURCE_COMPILER";
static SOURCE_COMPILER_ENV_LOCK: Mutex<()> = Mutex::new(());

fn register_agent(world: &mut World, agent_id: &str) {
    world.submit_action(Action::RegisterAgent {
        agent_id: agent_id.to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("register agent");
    world
        .set_agent_resource_balance(agent_id, ResourceKind::Electricity, 128)
        .expect("seed electricity");
    world
        .set_agent_resource_balance(agent_id, ResourceKind::Data, 64)
        .expect("seed data");
}

fn set_agent_resource(world: &mut World, agent_id: &str, kind: ResourceKind, amount: i64) {
    world
        .set_agent_resource_balance(agent_id, kind, amount)
        .expect("set agent resource balance");
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

fn sample_module_source_package() -> ModuleSourcePackage {
    ModuleSourcePackage {
        manifest_path: "Cargo.toml".to_string(),
        files: BTreeMap::from([
            (
                "Cargo.toml".to_string(),
                br#"[package]
name = "sample_module"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
"#
                .to_vec(),
            ),
            (
                "src/lib.rs".to_string(),
                b"#[no_mangle] pub extern \"C\" fn reduce() {}".to_vec(),
            ),
        ]),
    }
}

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration since epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "agent-world-runtime-tests-{prefix}-{}-{unique}",
        std::process::id()
    ))
}

fn write_fake_source_compiler(script_path: &Path, produced_wasm_bytes: &str) {
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nout_path=\"$4\"\nprintf '%s' '{produced_wasm_bytes}' > \"$out_path\"\n"
    );
    fs::write(script_path, script).expect("write fake source compiler");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(script_path, fs::Permissions::from_mode(0o755))
            .expect("chmod fake source compiler");
    }
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            previous: std::env::var(key).ok(),
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
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
        fee_kind,
        fee_amount,
    }) = &event.body
    else {
        panic!("expected module artifact deployed event: {:?}", event.body);
    };
    assert_eq!(publisher_agent_id, "publisher-1");
    assert_eq!(event_hash, &wasm_hash);
    assert_eq!(*bytes_len, wasm_bytes.len() as u64);
    assert_eq!(*fee_kind, ResourceKind::Electricity);
    assert!(*fee_amount > 0);

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
fn compile_module_artifact_from_source_registers_compiled_artifact() {
    let _env_lock = SOURCE_COMPILER_ENV_LOCK.lock().expect("lock compile env");
    let _env_guard = EnvVarGuard::capture(SOURCE_COMPILER_ENV);
    let temp_root = temp_dir("compile-module-artifact");
    fs::create_dir_all(&temp_root).expect("create temp dir");
    let compiler_script = temp_root.join("compiler.sh");
    let produced_wasm_bytes = "compiled-from-source-runtime";
    write_fake_source_compiler(compiler_script.as_path(), produced_wasm_bytes);
    std::env::set_var(SOURCE_COMPILER_ENV, compiler_script.as_os_str());

    let mut world = World::new();
    register_agent(&mut world, "publisher-1");

    world.submit_action(Action::CompileModuleArtifactFromSource {
        publisher_agent_id: "publisher-1".to_string(),
        module_id: "m.loop.source.compile".to_string(),
        source_package: sample_module_source_package(),
    });
    world.step().expect("compile from source action");

    let wasm_bytes = produced_wasm_bytes.as_bytes().to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    assert!(matches!(
        world.journal().events.last().map(|event| &event.body),
        Some(WorldEventBody::Domain(DomainEvent::ModuleArtifactDeployed {
            publisher_agent_id,
            wasm_hash: event_hash,
            bytes_len,
            ..
        })) if publisher_agent_id == "publisher-1" && event_hash == &wasm_hash && *bytes_len == wasm_bytes.len() as u64
    ));
    assert_eq!(
        world.state().module_artifact_owners.get(&wasm_hash),
        Some(&"publisher-1".to_string())
    );
    assert_eq!(
        world
            .load_module(&wasm_hash)
            .expect("load compiled module")
            .bytes,
        wasm_bytes
    );

    let _ = fs::remove_dir_all(temp_root);
}

#[test]
fn compile_module_artifact_from_source_rejects_when_manifest_path_missing_in_files() {
    let mut world = World::new();
    register_agent(&mut world, "publisher-1");

    let action_id = world.submit_action(Action::CompileModuleArtifactFromSource {
        publisher_agent_id: "publisher-1".to_string(),
        module_id: "m.loop.source.invalid".to_string(),
        source_package: ModuleSourcePackage {
            manifest_path: "Cargo.toml".to_string(),
            files: BTreeMap::from([(
                "src/lib.rs".to_string(),
                b"#[no_mangle] pub extern \"C\" fn reduce() {}".to_vec(),
            )]),
        },
    });
    world.step().expect("compile invalid source action");

    assert_last_rejection_note(&world, action_id, "manifest path missing");
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
        install_target,
        proposal_id,
        manifest_hash,
        fee_kind,
        fee_amount,
    }) = &event.body
    else {
        panic!("expected module installed event: {:?}", event.body);
    };
    assert_eq!(installer_agent_id, "installer-1");
    assert_eq!(module_id, "m.loop.active");
    assert_eq!(module_version, "0.1.0");
    assert!(*active);
    assert_eq!(*install_target, ModuleInstallTarget::SelfAgent);
    assert!(!manifest_hash.is_empty());
    assert_eq!(*fee_kind, ResourceKind::Electricity);
    assert!(*fee_amount > 0);

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

#[test]
fn module_artifact_listing_and_purchase_transfers_owner_and_settles_price() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "buyer-1");
    set_agent_resource(&mut world, "seller-1", ResourceKind::Hardware, 3);
    set_agent_resource(&mut world, "buyer-1", ResourceKind::Hardware, 20);

    let wasm_bytes = b"module-action-loop-market-list-and-buy".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::ListModuleArtifactForSale {
        seller_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 7,
    });
    world.step().expect("list artifact");
    assert!(matches!(
        world.journal().events.last().map(|event| &event.body),
        Some(WorldEventBody::Domain(DomainEvent::ModuleArtifactListed {
            seller_agent_id,
            wasm_hash: listed_hash,
            price_kind: ResourceKind::Hardware,
            price_amount: 7,
            ..
        })) if seller_agent_id == "seller-1" && listed_hash == &wasm_hash
    ));

    world.submit_action(Action::BuyModuleArtifact {
        buyer_agent_id: "buyer-1".to_string(),
        wasm_hash: wasm_hash.clone(),
    });
    world.step().expect("buy artifact");
    assert!(matches!(
        world.journal().events.last().map(|event| &event.body),
        Some(WorldEventBody::Domain(
            DomainEvent::ModuleArtifactSaleCompleted {
                buyer_agent_id,
                seller_agent_id,
                wasm_hash: sold_hash,
                price_kind: ResourceKind::Hardware,
                price_amount: 7,
                ..
            }
        )) if buyer_agent_id == "buyer-1" && seller_agent_id == "seller-1" && sold_hash == &wasm_hash
    ));

    assert_eq!(
        world.state().module_artifact_owners.get(&wasm_hash),
        Some(&"buyer-1".to_string())
    );
    assert!(!world
        .state()
        .module_artifact_listings
        .contains_key(&wasm_hash));
    assert_eq!(
        world
            .agent_resource_balance("buyer-1", ResourceKind::Hardware)
            .expect("buyer resource"),
        13
    );
    assert_eq!(
        world
            .agent_resource_balance("seller-1", ResourceKind::Hardware)
            .expect("seller resource"),
        10
    );
}

#[test]
fn list_module_artifact_for_sale_rejects_non_owner() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "intruder-1");

    let wasm_bytes = b"module-action-loop-list-non-owner".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    let action_id = world.submit_action(Action::ListModuleArtifactForSale {
        seller_agent_id: "intruder-1".to_string(),
        wasm_hash,
        price_kind: ResourceKind::Hardware,
        price_amount: 5,
    });
    world.step().expect("list non-owner");

    assert_last_rejection_note(&world, action_id, "does not own");
}

#[test]
fn buy_module_artifact_rejects_when_buyer_has_insufficient_price_resource() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "buyer-1");
    set_agent_resource(&mut world, "buyer-1", ResourceKind::Hardware, 2);

    let wasm_bytes = b"module-action-loop-buy-insufficient".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");
    world.submit_action(Action::ListModuleArtifactForSale {
        seller_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 5,
    });
    world.step().expect("list artifact");

    let action_id = world.submit_action(Action::BuyModuleArtifact {
        buyer_agent_id: "buyer-1".to_string(),
        wasm_hash,
    });
    world.step().expect("buy insufficient");

    let event = world.journal().events.last().expect("last event");
    let WorldEventBody::Domain(DomainEvent::ActionRejected {
        action_id: rejected_action_id,
        reason:
            RejectReason::InsufficientResource {
                agent_id,
                kind: ResourceKind::Hardware,
                requested,
                available,
            },
    }) = &event.body
    else {
        panic!(
            "expected insufficient resource rejection for buy action: {:?}",
            event.body
        );
    };
    assert_eq!(*rejected_action_id, action_id);
    assert_eq!(agent_id, "buyer-1");
    assert_eq!(*requested, 5);
    assert_eq!(*available, 2);
}

#[test]
fn install_module_from_artifact_rejects_non_owner_when_owner_is_registered() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "installer-1");

    let wasm_bytes = b"module-action-loop-install-owner-check".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    let action_id = world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "installer-1".to_string(),
        manifest: base_manifest("m.loop.owner-guard", "0.1.0", &wasm_hash),
        activate: true,
    });
    world.step().expect("install owner-guard");

    assert_last_rejection_note(&world, action_id, "does not own");
    assert!(world.module_registry().records.is_empty());
}

#[test]
fn delist_module_artifact_removes_listing_and_charges_data_fee() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");

    let wasm_bytes = b"module-action-loop-delist-success".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");
    world.submit_action(Action::ListModuleArtifactForSale {
        seller_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 5,
    });
    world.step().expect("list artifact");

    let before_data = world
        .agent_resource_balance("seller-1", ResourceKind::Data)
        .expect("seller data before delist");

    world.submit_action(Action::DelistModuleArtifact {
        seller_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
    });
    world.step().expect("delist artifact");

    let event = world.journal().events.last().expect("last event");
    let WorldEventBody::Domain(DomainEvent::ModuleArtifactDelisted {
        seller_agent_id,
        wasm_hash: delisted_hash,
        order_id: _,
        fee_kind,
        fee_amount,
    }) = &event.body
    else {
        panic!("expected module artifact delisted event: {:?}", event.body);
    };
    assert_eq!(seller_agent_id, "seller-1");
    assert_eq!(delisted_hash, &wasm_hash);
    assert_eq!(*fee_kind, ResourceKind::Data);
    assert!(*fee_amount > 0);
    assert!(!world
        .state()
        .module_artifact_listings
        .contains_key(&wasm_hash));
    let after_data = world
        .agent_resource_balance("seller-1", ResourceKind::Data)
        .expect("seller data after delist");
    assert_eq!(after_data, before_data - *fee_amount);
}

#[test]
fn destroy_module_artifact_removes_owner_and_artifact_bytes() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_bytes = b"module-action-loop-destroy-success".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    let before_electricity = world
        .agent_resource_balance("owner-1", ResourceKind::Electricity)
        .expect("owner electricity before destroy");

    world.submit_action(Action::DestroyModuleArtifact {
        owner_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        reason: "retire obsolete module".to_string(),
    });
    world.step().expect("destroy artifact");

    let (destroyed_hash, fee_kind, fee_amount) = {
        let event = world.journal().events.last().expect("last event");
        let WorldEventBody::Domain(DomainEvent::ModuleArtifactDestroyed {
            owner_agent_id,
            wasm_hash: destroyed_hash,
            reason,
            fee_kind,
            fee_amount,
        }) = &event.body
        else {
            panic!("expected module artifact destroyed event: {:?}", event.body);
        };
        assert_eq!(owner_agent_id, "owner-1");
        assert_eq!(reason, "retire obsolete module");
        (destroyed_hash.clone(), *fee_kind, *fee_amount)
    };
    assert_eq!(destroyed_hash, wasm_hash);
    assert_eq!(fee_kind, ResourceKind::Electricity);
    assert!(fee_amount > 0);
    assert!(!world
        .state()
        .module_artifact_owners
        .contains_key(&wasm_hash));
    assert!(!world
        .state()
        .module_artifact_listings
        .contains_key(&wasm_hash));
    assert!(world.load_module(&wasm_hash).is_err());
    let after_electricity = world
        .agent_resource_balance("owner-1", ResourceKind::Electricity)
        .expect("owner electricity after destroy");
    assert_eq!(after_electricity, before_electricity - fee_amount);
}

#[test]
fn destroy_module_artifact_rejects_when_artifact_is_used_by_active_module() {
    let mut world = World::new();
    register_agent(&mut world, "owner-1");

    let wasm_bytes = b"module-action-loop-destroy-active-guard".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "owner-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "owner-1".to_string(),
        manifest: base_manifest("m.loop.destroy-guard", "0.1.0", &wasm_hash),
        activate: true,
    });
    world.step().expect("install module");

    let action_id = world.submit_action(Action::DestroyModuleArtifact {
        owner_agent_id: "owner-1".to_string(),
        wasm_hash,
        reason: "cleanup".to_string(),
    });
    world.step().expect("destroy guarded artifact");

    assert_last_rejection_note(&world, action_id, "used by active module");
}

#[test]
fn module_artifact_bid_auto_matches_on_listing() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "buyer-1");
    set_agent_resource(&mut world, "buyer-1", ResourceKind::Hardware, 30);

    let wasm_bytes = b"module-action-loop-bid-auto-match".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::PlaceModuleArtifactBid {
        bidder_agent_id: "buyer-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 9,
    });
    world.step().expect("place bid");

    let bid_order_id = match &world.journal().events.last().expect("bid event").body {
        WorldEventBody::Domain(DomainEvent::ModuleArtifactBidPlaced { order_id, .. }) => *order_id,
        other => panic!("expected module artifact bid placed event: {other:?}"),
    };
    assert!(bid_order_id > 0);

    world.submit_action(Action::ListModuleArtifactForSale {
        seller_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 7,
    });
    world.step().expect("list and match");

    let event = world.journal().events.last().expect("sale event");
    let WorldEventBody::Domain(DomainEvent::ModuleArtifactSaleCompleted {
        buyer_agent_id,
        seller_agent_id,
        price_kind,
        price_amount,
        bid_order_id: matched_bid_order_id,
        ..
    }) = &event.body
    else {
        panic!(
            "expected module artifact sale completed event for auto match: {:?}",
            event.body
        );
    };
    assert_eq!(buyer_agent_id, "buyer-1");
    assert_eq!(seller_agent_id, "seller-1");
    assert_eq!(*price_kind, ResourceKind::Hardware);
    assert_eq!(*price_amount, 7);
    assert_eq!(*matched_bid_order_id, Some(bid_order_id));
    assert_eq!(
        world.state().module_artifact_owners.get(&wasm_hash),
        Some(&"buyer-1".to_string())
    );
    assert!(!world
        .state()
        .module_artifact_listings
        .contains_key(&wasm_hash));
    assert!(!world.state().module_artifact_bids.contains_key(&wasm_hash));
}

#[test]
fn cancel_module_artifact_bid_removes_order() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "buyer-1");
    set_agent_resource(&mut world, "buyer-1", ResourceKind::Hardware, 20);

    let wasm_bytes = b"module-action-loop-bid-cancel".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::PlaceModuleArtifactBid {
        bidder_agent_id: "buyer-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 8,
    });
    world.step().expect("place bid");
    let bid_order_id = match &world.journal().events.last().expect("bid event").body {
        WorldEventBody::Domain(DomainEvent::ModuleArtifactBidPlaced { order_id, .. }) => *order_id,
        other => panic!("expected module artifact bid placed event: {other:?}"),
    };

    world.submit_action(Action::CancelModuleArtifactBid {
        bidder_agent_id: "buyer-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        bid_order_id,
    });
    world.step().expect("cancel bid");

    assert!(matches!(
        world.journal().events.last().map(|event| &event.body),
        Some(WorldEventBody::Domain(DomainEvent::ModuleArtifactBidCancelled {
            bidder_agent_id,
            order_id,
            ..
        })) if bidder_agent_id == "buyer-1" && *order_id == bid_order_id
    ));
    assert!(!world.state().module_artifact_bids.contains_key(&wasm_hash));
}

#[test]
fn module_artifact_bid_match_prefers_highest_price() {
    let mut world = World::new();
    register_agent(&mut world, "seller-1");
    register_agent(&mut world, "buyer-low");
    register_agent(&mut world, "buyer-high");
    set_agent_resource(&mut world, "buyer-low", ResourceKind::Hardware, 20);
    set_agent_resource(&mut world, "buyer-high", ResourceKind::Hardware, 20);

    let wasm_bytes = b"module-action-loop-bid-priority".to_vec();
    let wasm_hash = util::sha256_hex(&wasm_bytes);
    world.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
    });
    world.step().expect("deploy artifact");

    world.submit_action(Action::PlaceModuleArtifactBid {
        bidder_agent_id: "buyer-low".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 8,
    });
    world.step().expect("place low bid");
    world.submit_action(Action::PlaceModuleArtifactBid {
        bidder_agent_id: "buyer-high".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 9,
    });
    world.step().expect("place high bid");

    world.submit_action(Action::ListModuleArtifactForSale {
        seller_agent_id: "seller-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        price_kind: ResourceKind::Hardware,
        price_amount: 7,
    });
    world.step().expect("list and match");

    let event = world.journal().events.last().expect("sale event");
    let WorldEventBody::Domain(DomainEvent::ModuleArtifactSaleCompleted { buyer_agent_id, .. }) =
        &event.body
    else {
        panic!("expected sale completion event: {:?}", event.body);
    };
    assert_eq!(buyer_agent_id, "buyer-high");
}
