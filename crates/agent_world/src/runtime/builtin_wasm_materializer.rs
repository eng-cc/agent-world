use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::{util, BlobStore, HashAlgorithm, LocalCasStore, WorldError};

const BUILTIN_WASM_FETCHER_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_FETCHER";
const BUILTIN_WASM_FETCH_URLS_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_FETCH_URLS";
const BUILTIN_WASM_COMPILER_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_COMPILER";
const BUILTIN_WASM_FETCH_TIMEOUT_MS_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_FETCH_TIMEOUT_MS";

const DEFAULT_FETCH_TIMEOUT_MS: u64 = 1_500;
const BUILTIN_WASM_BUILD_PROFILE: &str = "release";
const BUILTIN_WASM_BUILD_TARGET: &str = "wasm32-unknown-unknown";

pub(crate) fn load_builtin_wasm_with_fetch_fallback(
    module_id: &str,
    expected_hash: &str,
    distfs_root: &Path,
) -> Result<Vec<u8>, WorldError> {
    let store = LocalCasStore::new_with_hash_algorithm(distfs_root, HashAlgorithm::Sha256);
    if let Ok(bytes) = store.get_verified(expected_hash) {
        return Ok(bytes);
    }

    if let Some(fetched) = try_fetch_builtin_wasm(module_id, expected_hash)? {
        store.put(expected_hash, &fetched)?;
        return store.get_verified(expected_hash).map_err(WorldError::from);
    }

    let compiled = compile_builtin_wasm(module_id, expected_hash)?;
    store.put(expected_hash, &compiled)?;
    store.get_verified(expected_hash).map_err(WorldError::from)
}

fn try_fetch_builtin_wasm(
    module_id: &str,
    expected_hash: &str,
) -> Result<Option<Vec<u8>>, WorldError> {
    if let Some(bytes) = try_fetch_via_fetcher(module_id, expected_hash)? {
        return Ok(Some(bytes));
    }
    try_fetch_via_http(expected_hash)
}

fn try_fetch_via_fetcher(
    module_id: &str,
    expected_hash: &str,
) -> Result<Option<Vec<u8>>, WorldError> {
    let Some(fetcher_path) = env_non_empty(BUILTIN_WASM_FETCHER_ENV) else {
        return Ok(None);
    };
    let out_path = temp_artifact_path("fetched", module_id);
    let Some(parent) = out_path.parent() else {
        return Ok(None);
    };
    fs::create_dir_all(parent)?;

    let status = match Command::new(fetcher_path)
        .arg(module_id)
        .arg(expected_hash)
        .arg(&out_path)
        .status()
    {
        Ok(status) => status,
        Err(_) => return Ok(None),
    };
    if !status.success() {
        return Ok(None);
    }

    let bytes = match fs::read(&out_path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };
    if util::sha256_hex(&bytes) != expected_hash {
        return Ok(None);
    }
    Ok(Some(bytes))
}

fn try_fetch_via_http(expected_hash: &str) -> Result<Option<Vec<u8>>, WorldError> {
    let Some(fetch_urls) = env_non_empty(BUILTIN_WASM_FETCH_URLS_ENV) else {
        return Ok(None);
    };
    let timeout = fetch_timeout();
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| WorldError::Io(error.to_string()))?;

    for base in fetch_urls
        .split(',')
        .map(str::trim)
        .filter(|url| !url.is_empty())
    {
        let trimmed = base.trim_end_matches('/');
        let candidates = [
            format!("{trimmed}/{expected_hash}.blob"),
            format!("{trimmed}/{expected_hash}"),
        ];
        for url in candidates {
            let Ok(response) = client.get(&url).send() else {
                continue;
            };
            if !response.status().is_success() {
                continue;
            }
            let Ok(bytes) = response.bytes() else {
                continue;
            };
            if util::sha256_hex(bytes.as_ref()) == expected_hash {
                return Ok(Some(bytes.to_vec()));
            }
        }
    }
    Ok(None)
}

fn compile_builtin_wasm(module_id: &str, expected_hash: &str) -> Result<Vec<u8>, WorldError> {
    if let Some(compiler_path) = env_non_empty(BUILTIN_WASM_COMPILER_ENV) {
        return compile_via_command(Path::new(&compiler_path), module_id, expected_hash);
    }
    compile_via_default_script(module_id, expected_hash)
}

fn compile_via_command(
    compiler_path: &Path,
    module_id: &str,
    expected_hash: &str,
) -> Result<Vec<u8>, WorldError> {
    let out_path = temp_artifact_path("compiled", module_id);
    let Some(parent) = out_path.parent() else {
        return Err(WorldError::ModuleChangeInvalid {
            reason: "compiler output path has no parent".to_string(),
        });
    };
    fs::create_dir_all(parent)?;

    let status = Command::new(compiler_path)
        .arg(module_id)
        .arg(expected_hash)
        .arg(&out_path)
        .status()
        .map_err(|error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "failed to execute builtin wasm compiler={} err={error}",
                compiler_path.display()
            ),
        })?;

    if !status.success() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "builtin wasm compiler exited non-zero compiler={} status={status}",
                compiler_path.display()
            ),
        });
    }

    let bytes = fs::read(&out_path).map_err(|error| WorldError::ModuleChangeInvalid {
        reason: format!(
            "builtin wasm compiler output missing module_id={module_id} out={} err={error}",
            out_path.display()
        ),
    })?;

    validate_compiled_hash(module_id, expected_hash, &bytes)?;
    Ok(bytes)
}

fn compile_via_default_script(module_id: &str, expected_hash: &str) -> Result<Vec<u8>, WorldError> {
    let repo_root = repo_root();
    let build_script = repo_root.join("scripts").join("build-wasm-module.sh");
    let manifest_path = repo_root
        .join("crates")
        .join("agent_world_builtin_wasm")
        .join("Cargo.toml");
    let out_dir = temp_build_dir(module_id);
    fs::create_dir_all(&out_dir)?;

    let status = Command::new(&build_script)
        .arg("--module-id")
        .arg(module_id)
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--profile")
        .arg(BUILTIN_WASM_BUILD_PROFILE)
        .arg("--target")
        .arg(BUILTIN_WASM_BUILD_TARGET)
        .status()
        .map_err(|error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "failed to execute fallback build script={} err={error}",
                build_script.display()
            ),
        })?;

    if !status.success() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "fallback build script exited non-zero script={} status={status}",
                build_script.display()
            ),
        });
    }

    let artifact_path = out_dir.join(format!("{module_id}.wasm"));
    let bytes = fs::read(&artifact_path).map_err(|error| WorldError::ModuleChangeInvalid {
        reason: format!(
            "fallback built artifact missing module_id={module_id} path={} err={error}",
            artifact_path.display()
        ),
    })?;
    validate_compiled_hash(module_id, expected_hash, &bytes)?;
    let _ = fs::remove_dir_all(&out_dir);
    Ok(bytes)
}

fn validate_compiled_hash(
    module_id: &str,
    expected_hash: &str,
    bytes: &[u8],
) -> Result<(), WorldError> {
    let actual = util::sha256_hex(bytes);
    if actual != expected_hash {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "fallback compile hash mismatch module_id={module_id} expected={expected_hash} actual={actual}",
            ),
        });
    }
    Ok(())
}

fn fetch_timeout() -> Duration {
    let timeout_ms = env_non_empty(BUILTIN_WASM_FETCH_TIMEOUT_MS_ENV)
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_FETCH_TIMEOUT_MS);
    Duration::from_millis(timeout_ms)
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn temp_artifact_path(kind: &str, module_id: &str) -> PathBuf {
    temp_build_dir(module_id).join(format!("{module_id}.{kind}.wasm"))
}

fn temp_build_dir(module_id: &str) -> PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "agent-world-builtin-wasm-{module_id}-{}-{now}",
        std::process::id()
    ))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}
