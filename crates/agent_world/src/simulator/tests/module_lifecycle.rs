use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

const SOURCE_COMPILER_ENV: &str = "AGENT_WORLD_MODULE_SOURCE_COMPILER";

fn source_compiler_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
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

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn setup_kernel_with_agent(agent_id: &str) -> WorldKernel {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-home".to_string(),
        name: "home".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: agent_id.to_string(),
        location_id: "loc-home".to_string(),
    });
    kernel.step_until_empty();
    kernel
}

fn write_fake_source_compiler(script_path: &Path, output_wasm_text: &str) {
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nout_path=\"$4\"\nmkdir -p \"$(dirname \"$out_path\")\"\nprintf '%s' '{}' > \"$out_path\"\n",
        output_wasm_text
    );
    fs::write(script_path, script).expect("write fake compiler script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(script_path)
            .expect("script metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(script_path, permissions).expect("set executable permission");
    }
}

#[test]
fn module_lifecycle_deploy_and_install_succeeds_for_owner() {
    let mut kernel = setup_kernel_with_agent("agent-1");
    let wasm_bytes = b"simulator-module-lifecycle".to_vec();
    let wasm_hash = sha256_hex(wasm_bytes.as_slice());

    kernel.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "agent-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes: wasm_bytes.clone(),
        module_id_hint: Some("m.sim.lifecycle".to_string()),
    });
    let deploy_event = kernel.step().expect("deploy event");
    match deploy_event.kind {
        WorldEventKind::ModuleArtifactDeployed {
            publisher_agent_id,
            wasm_hash: event_hash,
            bytes_len,
            module_id_hint,
            ..
        } => {
            assert_eq!(publisher_agent_id, "agent-1");
            assert_eq!(event_hash, wasm_hash);
            assert_eq!(bytes_len, wasm_bytes.len() as u64);
            assert_eq!(module_id_hint.as_deref(), Some("m.sim.lifecycle"));
        }
        other => panic!("unexpected deploy event: {other:?}"),
    }

    let artifact = kernel
        .model()
        .module_artifacts
        .get(&wasm_hash)
        .expect("artifact exists");
    assert_eq!(artifact.publisher_agent_id, "agent-1");
    assert_eq!(artifact.wasm_bytes, wasm_bytes);

    kernel.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "agent-1".to_string(),
        module_id: "m.sim.lifecycle".to_string(),
        module_version: "0.1.0".to_string(),
        wasm_hash: wasm_hash.clone(),
        activate: true,
    });
    let install_event = kernel.step().expect("install event");
    match install_event.kind {
        WorldEventKind::ModuleInstalled {
            installer_agent_id,
            module_id,
            module_version,
            wasm_hash: event_hash,
            active,
        } => {
            assert_eq!(installer_agent_id, "agent-1");
            assert_eq!(module_id, "m.sim.lifecycle");
            assert_eq!(module_version, "0.1.0");
            assert_eq!(event_hash, wasm_hash);
            assert!(active);
        }
        other => panic!("unexpected install event: {other:?}"),
    }

    let installed = kernel
        .model()
        .installed_modules
        .get("m.sim.lifecycle")
        .expect("installed module exists");
    assert_eq!(installed.wasm_hash, wasm_hash);
    assert!(installed.active);
}

#[test]
fn module_lifecycle_install_rejects_non_owner() {
    let mut kernel = setup_kernel_with_agent("agent-owner");
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-other".to_string(),
        location_id: "loc-home".to_string(),
    });
    kernel.step().expect("register second agent");

    let wasm_bytes = b"simulator-module-non-owner".to_vec();
    let wasm_hash = sha256_hex(wasm_bytes.as_slice());
    kernel.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "agent-owner".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
        module_id_hint: None,
    });
    let _ = kernel.step().expect("deploy event");

    kernel.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "agent-other".to_string(),
        module_id: "m.sim.owner-check".to_string(),
        module_version: "0.1.0".to_string(),
        wasm_hash,
        activate: true,
    });
    let event = kernel.step().expect("install reject event");
    match event.kind {
        WorldEventKind::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
        } => {
            assert!(
                notes.iter().any(|note| note.contains("not artifact owner")),
                "unexpected notes: {notes:?}"
            );
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn module_lifecycle_compile_from_source_deploys_artifact() {
    let _lock = source_compiler_env_lock().lock().expect("env lock");
    let _guard = EnvVarGuard::capture(SOURCE_COMPILER_ENV);
    let temp_root = std::env::temp_dir().join(format!(
        "agent-world-sim-compile-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("create temp root");

    let script_path = temp_root.join("compiler.sh");
    let compiled_text = "sim-compiled-wasm";
    write_fake_source_compiler(script_path.as_path(), compiled_text);
    std::env::set_var(SOURCE_COMPILER_ENV, script_path.as_os_str());

    let mut kernel = setup_kernel_with_agent("agent-1");
    kernel.submit_action(Action::CompileModuleArtifactFromSource {
        publisher_agent_id: "agent-1".to_string(),
        module_id: "m.sim.compile".to_string(),
        manifest_path: "Cargo.toml".to_string(),
        source_files: BTreeMap::from([
            (
                "Cargo.toml".to_string(),
                br#"[package]
name = \"m_sim_compile\"
version = \"0.1.0\"
edition = \"2021\""#
                    .to_vec(),
            ),
            (
                "src/lib.rs".to_string(),
                b"#[no_mangle] pub extern \"C\" fn tick() {}".to_vec(),
            ),
        ]),
    });

    let event = kernel.step().expect("compile event");
    let expected_bytes = compiled_text.as_bytes().to_vec();
    let expected_hash = sha256_hex(expected_bytes.as_slice());
    match event.kind {
        WorldEventKind::ModuleArtifactDeployed {
            publisher_agent_id,
            wasm_hash,
            bytes_len,
            module_id_hint,
            wasm_bytes,
        } => {
            assert_eq!(publisher_agent_id, "agent-1");
            assert_eq!(wasm_hash, expected_hash);
            assert_eq!(bytes_len, expected_bytes.len() as u64);
            assert_eq!(module_id_hint.as_deref(), Some("m.sim.compile"));
            assert_eq!(wasm_bytes, expected_bytes);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let _ = fs::remove_dir_all(temp_root);
}

#[test]
fn module_lifecycle_replay_restores_artifact_and_install_state() {
    let mut kernel = setup_kernel_with_agent("agent-1");
    let snapshot = kernel.snapshot();

    let wasm_bytes = b"sim-replay-module".to_vec();
    let wasm_hash = sha256_hex(wasm_bytes.as_slice());
    kernel.submit_action(Action::DeployModuleArtifact {
        publisher_agent_id: "agent-1".to_string(),
        wasm_hash: wasm_hash.clone(),
        wasm_bytes,
        module_id_hint: Some("m.sim.replay".to_string()),
    });
    kernel.step().expect("deploy event");
    kernel.submit_action(Action::InstallModuleFromArtifact {
        installer_agent_id: "agent-1".to_string(),
        module_id: "m.sim.replay".to_string(),
        module_version: "0.2.0".to_string(),
        wasm_hash: wasm_hash.clone(),
        activate: true,
    });
    kernel.step().expect("install event");

    let journal = kernel.journal_snapshot();
    let replayed =
        WorldKernel::replay_from_snapshot(snapshot, journal).expect("replay from snapshot");

    assert!(replayed.model().module_artifacts.contains_key(&wasm_hash));
    let installed = replayed
        .model()
        .installed_modules
        .get("m.sim.replay")
        .expect("installed module present after replay");
    assert_eq!(installed.module_version, "0.2.0");
    assert!(installed.active);
}
