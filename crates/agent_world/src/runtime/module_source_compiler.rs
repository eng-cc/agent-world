use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use super::{ModuleSourcePackage, WorldError};

const MODULE_SOURCE_COMPILER_ENV: &str = "AGENT_WORLD_MODULE_SOURCE_COMPILER";
const MODULE_SOURCE_BUILD_PROFILE: &str = "release";

pub(crate) fn compile_module_artifact_from_source(
    module_id: &str,
    source_package: &ModuleSourcePackage,
) -> Result<Vec<u8>, WorldError> {
    if module_id.trim().is_empty() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: "compile module source rejected: module_id is empty".to_string(),
        });
    }
    if source_package.manifest_path.trim().is_empty() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: "compile module source rejected: manifest_path is empty".to_string(),
        });
    }
    if source_package.files.is_empty() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: "compile module source rejected: source files are empty".to_string(),
        });
    }

    let workspace_dir = temp_source_workspace(module_id);
    fs::create_dir_all(&workspace_dir).map_err(WorldError::from)?;

    let result = (|| {
        let manifest_rel = validate_relative_source_path(source_package.manifest_path.as_str())?;

        for (file_path, file_bytes) in &source_package.files {
            let rel_path = validate_relative_source_path(file_path.as_str())?;
            let full_path = workspace_dir.join(rel_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(WorldError::from)?;
            }
            fs::write(full_path, file_bytes).map_err(WorldError::from)?;
        }

        let manifest_abs = workspace_dir.join(manifest_rel.as_path());
        if !manifest_abs.exists() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "compile module source rejected: manifest path missing {}",
                    source_package.manifest_path
                ),
            });
        }

        let out_dir = workspace_dir.join("out");
        fs::create_dir_all(&out_dir).map_err(WorldError::from)?;
        let out_path = out_dir.join(format!("{module_id}.wasm"));

        if let Some(compiler_path) = env_non_empty(MODULE_SOURCE_COMPILER_ENV) {
            compile_via_custom_command(
                Path::new(compiler_path.as_str()),
                module_id,
                workspace_dir.as_path(),
                source_package.manifest_path.as_str(),
                out_path.as_path(),
            )?;
        } else {
            compile_via_default_script(
                module_id,
                workspace_dir.as_path(),
                source_package.manifest_path.as_str(),
                out_dir.as_path(),
            )?;
        }

        let output_bytes =
            fs::read(out_path.as_path()).map_err(|error| WorldError::ModuleChangeInvalid {
                reason: format!(
                    "compile module source rejected: output missing path={} err={error}",
                    out_path.display()
                ),
            })?;
        Ok(output_bytes)
    })();

    let _ = fs::remove_dir_all(&workspace_dir);
    result
}

fn compile_via_custom_command(
    compiler_path: &Path,
    module_id: &str,
    workspace_dir: &Path,
    manifest_path: &str,
    out_path: &Path,
) -> Result<(), WorldError> {
    let status = Command::new(compiler_path)
        .arg(module_id)
        .arg(workspace_dir)
        .arg(manifest_path)
        .arg(out_path)
        .status()
        .map_err(|error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "compile module source rejected: execute compiler failed compiler={} err={error}",
                compiler_path.display()
            ),
        })?;

    if status.success() {
        return Ok(());
    }

    Err(WorldError::ModuleChangeInvalid {
        reason: format!(
            "compile module source rejected: compiler exited non-zero compiler={} status={status}",
            compiler_path.display()
        ),
    })
}

fn compile_via_default_script(
    module_id: &str,
    workspace_dir: &Path,
    manifest_path: &str,
    out_dir: &Path,
) -> Result<(), WorldError> {
    let script_path = repo_root().join("scripts").join("build-wasm-module.sh");
    let manifest_abs = workspace_dir.join(manifest_path);

    let status = Command::new(script_path.as_path())
        .arg("--module-id")
        .arg(module_id)
        .arg("--manifest-path")
        .arg(manifest_abs.as_path())
        .arg("--out-dir")
        .arg(out_dir)
        .arg("--profile")
        .arg(MODULE_SOURCE_BUILD_PROFILE)
        .status()
        .map_err(|error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "compile module source rejected: execute default builder failed script={} err={error}",
                script_path.display()
            ),
        })?;

    if status.success() {
        return Ok(());
    }

    Err(WorldError::ModuleChangeInvalid {
        reason: format!(
            "compile module source rejected: default builder exited non-zero script={} status={status}",
            script_path.display()
        ),
    })
}

fn validate_relative_source_path(raw: &str) -> Result<PathBuf, WorldError> {
    let path = Path::new(raw);
    if path.as_os_str().is_empty() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: "compile module source rejected: source path is empty".to_string(),
        });
    }
    if path.is_absolute() {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "compile module source rejected: absolute source path is not allowed {}",
                raw
            ),
        });
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "compile module source rejected: invalid source path {}",
                        raw
                    ),
                });
            }
        }
    }
    Ok(path.to_path_buf())
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn temp_source_workspace(module_id: &str) -> PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "agent-world-module-source-{module_id}-{}-{now}",
        std::process::id()
    ))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}
