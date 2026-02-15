#![cfg(feature = "test_tier_full")]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::{
    load_builtin_wasm_with_fetch_fallback, util, BlobStore, HashAlgorithm, LocalCasStore,
};

const FETCHER_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_FETCHER";
const DISTFS_ROOT_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_DISTFS_ROOT";

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn materializer_fetch_miss_falls_back_to_compile_and_caches_blob() {
    let _env_guard = ENV_LOCK.lock().expect("lock env");
    let temp_root = temp_dir("builtin-materializer");
    fs::create_dir_all(&temp_root).expect("create temp root");

    let distfs_root = temp_root.join("distfs");
    fs::create_dir_all(distfs_root.join("blobs")).expect("create distfs blobs");

    let fetch_log = temp_root.join("fetch.log");
    let fetcher = temp_root.join("fetcher.sh");
    write_fetcher_script(&fetcher, &fetch_log);

    let module_id = "m1.rule.move";
    let expected_hash = manifest_hash_for_module(module_id).expect("manifest hash");

    let _fetcher_guard = EnvVarGuard::capture(FETCHER_ENV);
    let _distfs_guard = EnvVarGuard::capture(DISTFS_ROOT_ENV);
    std::env::set_var(FETCHER_ENV, &fetcher);
    std::env::set_var(DISTFS_ROOT_ENV, &distfs_root);

    let load_result =
        load_builtin_wasm_with_fetch_fallback(module_id, &expected_hash, &distfs_root);

    let bytes = load_result.expect("load builtin wasm");
    assert_eq!(util::sha256_hex(&bytes), expected_hash);

    let fetched_log = fs::read_to_string(&fetch_log).expect("read fetch log");
    assert!(
        fetched_log.contains(module_id),
        "fetcher log should contain module_id"
    );
    assert!(
        fetched_log.contains(&expected_hash),
        "fetcher log should contain expected hash"
    );

    let store = LocalCasStore::new_with_hash_algorithm(&distfs_root, HashAlgorithm::Sha256);
    assert!(store.has(&expected_hash).expect("distfs has expected hash"));
    let cached = store
        .get_verified(&expected_hash)
        .expect("verified distfs blob");
    assert_eq!(cached, bytes);

    let _ = fs::remove_dir_all(&temp_root);
}

fn write_fetcher_script(script_path: &Path, fetch_log: &Path) {
    let script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\necho \"$1 $2\" >> \"{}\"\nexit 1\n",
        fetch_log.display()
    );
    fs::write(script_path, script).expect("write fetcher script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(script_path, permissions).expect("chmod fetcher script");
    }
}

fn manifest_hash_for_module(module_id: &str) -> Option<String> {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("runtime")
        .join("world")
        .join("artifacts")
        .join("m1_builtin_modules.sha256");
    let content = fs::read_to_string(manifest_path).ok()?;
    content.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let id = parts.next()?;
        let hash = parts.next()?;
        if id == module_id {
            Some(hash.to_string())
        } else {
            None
        }
    })
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
