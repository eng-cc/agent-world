use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_TARGET: &str = "wasm32-unknown-unknown";
pub const DEFAULT_PROFILE: &str = "release";
pub const DEFAULT_OUT_DIR: &str = ".tmp/wasm-build-suite";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRequest {
    pub module_id: String,
    pub manifest_path: PathBuf,
    pub out_dir: PathBuf,
    pub target: String,
    pub profile: String,
    pub dry_run: bool,
}

impl BuildRequest {
    pub fn with_defaults(module_id: impl Into<String>, manifest_path: impl Into<PathBuf>) -> Self {
        Self {
            module_id: module_id.into(),
            manifest_path: manifest_path.into(),
            out_dir: PathBuf::from(DEFAULT_OUT_DIR),
            target: DEFAULT_TARGET.to_string(),
            profile: DEFAULT_PROFILE.to_string(),
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub module_id: String,
    pub target: String,
    pub profile: String,
    pub source_manifest_path: String,
    pub source_artifact_path: String,
    pub packaged_wasm_path: String,
    pub wasm_hash_sha256: String,
    pub wasm_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutput {
    pub module_id: String,
    pub source_artifact_path: PathBuf,
    pub packaged_wasm_path: PathBuf,
    pub metadata_path: PathBuf,
    pub wasm_hash_sha256: Option<String>,
    pub wasm_size_bytes: Option<u64>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub enum BuildError {
    InvalidArgument(String),
    CommandFailed {
        program: String,
        args: Vec<String>,
        status_code: Option<i32>,
        stderr: String,
    },
    Io {
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    Json {
        source: serde_json::Error,
        context: String,
    },
    ManifestNotFound(PathBuf),
    MetadataInvalid(String),
    ArtifactNotFound(PathBuf),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
            BuildError::CommandFailed {
                program,
                args,
                status_code,
                stderr,
            } => {
                write!(
                    f,
                    "command failed: {} {} (status={:?}){}",
                    program,
                    args.join(" "),
                    status_code,
                    if stderr.is_empty() {
                        String::new()
                    } else {
                        format!(", stderr={stderr}")
                    }
                )
            }
            BuildError::Io { path, source } => {
                if let Some(path) = path {
                    write!(f, "io error at {}: {}", path.display(), source)
                } else {
                    write!(f, "io error: {source}")
                }
            }
            BuildError::Json { source, context } => {
                write!(f, "json error ({context}): {source}")
            }
            BuildError::ManifestNotFound(path) => {
                write!(f, "manifest not found: {}", path.display())
            }
            BuildError::MetadataInvalid(msg) => write!(f, "cargo metadata invalid: {msg}"),
            BuildError::ArtifactNotFound(path) => {
                write!(f, "wasm artifact not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for BuildError {}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    target_directory: String,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    manifest_path: String,
    targets: Vec<CargoTarget>,
}

#[derive(Debug, Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
}

pub fn run_build(request: &BuildRequest) -> Result<BuildOutput, BuildError> {
    validate_request(request)?;
    let manifest_path = canonical_or_original(&request.manifest_path);
    if !manifest_path.exists() {
        return Err(BuildError::ManifestNotFound(manifest_path));
    }

    let metadata = read_cargo_metadata(&manifest_path)?;
    let package = find_package_for_manifest(&metadata, &manifest_path)?;
    let target_name = find_wasm_target_name(package)?;
    let artifact_path = resolve_artifact_path(
        &metadata,
        request.target.as_str(),
        request.profile.as_str(),
        target_name.as_str(),
    );

    let packaged_wasm_path = request
        .out_dir
        .join(format!("{}.wasm", request.module_id.as_str()));
    let metadata_path = request
        .out_dir
        .join(format!("{}.metadata.json", request.module_id.as_str()));

    if request.dry_run {
        return Ok(BuildOutput {
            module_id: request.module_id.clone(),
            source_artifact_path: artifact_path,
            packaged_wasm_path,
            metadata_path,
            wasm_hash_sha256: None,
            wasm_size_bytes: None,
            dry_run: true,
        });
    }

    run_cargo_build(
        manifest_path.as_path(),
        request.target.as_str(),
        request.profile.as_str(),
    )?;

    if !artifact_path.exists() {
        return Err(BuildError::ArtifactNotFound(artifact_path));
    }
    let wasm_bytes = fs::read(&artifact_path).map_err(|source| BuildError::Io {
        path: Some(artifact_path.clone()),
        source,
    })?;
    let wasm_size_bytes = u64::try_from(wasm_bytes.len()).map_err(|_| {
        BuildError::MetadataInvalid("wasm size overflow while converting usize to u64".to_string())
    })?;
    let wasm_hash_sha256 = sha256_hex(&wasm_bytes);

    if let Some(parent) = packaged_wasm_path.parent() {
        fs::create_dir_all(parent).map_err(|source| BuildError::Io {
            path: Some(parent.to_path_buf()),
            source,
        })?;
    }

    fs::copy(&artifact_path, &packaged_wasm_path).map_err(|source| BuildError::Io {
        path: Some(packaged_wasm_path.clone()),
        source,
    })?;

    let metadata_payload = BuildMetadata {
        module_id: request.module_id.clone(),
        target: request.target.clone(),
        profile: request.profile.clone(),
        source_manifest_path: manifest_path.to_string_lossy().to_string(),
        source_artifact_path: artifact_path.to_string_lossy().to_string(),
        packaged_wasm_path: packaged_wasm_path.to_string_lossy().to_string(),
        wasm_hash_sha256: wasm_hash_sha256.clone(),
        wasm_size_bytes,
    };

    let metadata_json =
        serde_json::to_vec_pretty(&metadata_payload).map_err(|source| BuildError::Json {
            source,
            context: "serialize build metadata".to_string(),
        })?;
    fs::write(&metadata_path, metadata_json).map_err(|source| BuildError::Io {
        path: Some(metadata_path.clone()),
        source,
    })?;

    Ok(BuildOutput {
        module_id: request.module_id.clone(),
        source_artifact_path: artifact_path,
        packaged_wasm_path,
        metadata_path,
        wasm_hash_sha256: Some(wasm_hash_sha256),
        wasm_size_bytes: Some(wasm_size_bytes),
        dry_run: false,
    })
}

fn validate_request(request: &BuildRequest) -> Result<(), BuildError> {
    if request.module_id.trim().is_empty() {
        return Err(BuildError::InvalidArgument(
            "module_id is empty".to_string(),
        ));
    }
    if request.target.trim().is_empty() {
        return Err(BuildError::InvalidArgument("target is empty".to_string()));
    }
    if request.profile != "release" && request.profile != "dev" {
        return Err(BuildError::InvalidArgument(format!(
            "profile must be release or dev, got {}",
            request.profile
        )));
    }
    if request.manifest_path.as_os_str().is_empty() {
        return Err(BuildError::InvalidArgument(
            "manifest_path is empty".to_string(),
        ));
    }
    Ok(())
}

fn read_cargo_metadata(manifest_path: &Path) -> Result<CargoMetadata, BuildError> {
    let args = vec![
        "metadata".to_string(),
        "--manifest-path".to_string(),
        manifest_path.to_string_lossy().to_string(),
        "--format-version".to_string(),
        "1".to_string(),
        "--no-deps".to_string(),
    ];
    let output = run_command_capture("cargo", args.as_slice())?;
    serde_json::from_slice(&output.stdout).map_err(|source| BuildError::Json {
        source,
        context: "parse cargo metadata output".to_string(),
    })
}

fn find_package_for_manifest<'a>(
    metadata: &'a CargoMetadata,
    manifest_path: &Path,
) -> Result<&'a CargoPackage, BuildError> {
    let canonical_manifest = canonical_or_original(manifest_path);
    metadata
        .packages
        .iter()
        .find(|package| {
            canonical_or_original(Path::new(package.manifest_path.as_str())) == canonical_manifest
        })
        .or_else(|| metadata.packages.first())
        .ok_or_else(|| {
            BuildError::MetadataInvalid("no package found in cargo metadata output".to_string())
        })
}

fn find_wasm_target_name(package: &CargoPackage) -> Result<String, BuildError> {
    let cdylib = package
        .targets
        .iter()
        .find(|target| target.kind.iter().any(|kind| kind == "cdylib"));
    if let Some(target) = cdylib {
        return Ok(normalize_artifact_name(target.name.as_str()));
    }

    let lib = package
        .targets
        .iter()
        .find(|target| target.kind.iter().any(|kind| kind == "lib"));
    if let Some(target) = lib {
        return Ok(normalize_artifact_name(target.name.as_str()));
    }

    Err(BuildError::MetadataInvalid(
        "no lib/cdylib target found in package; ensure the crate exports a library target"
            .to_string(),
    ))
}

fn resolve_artifact_path(
    metadata: &CargoMetadata,
    target: &str,
    profile: &str,
    target_name: &str,
) -> PathBuf {
    let profile_dir = match profile {
        "release" => "release",
        "dev" => "debug",
        other => other,
    };
    PathBuf::from(metadata.target_directory.as_str())
        .join(target)
        .join(profile_dir)
        .join(format!("{target_name}.wasm"))
}

fn run_cargo_build(manifest_path: &Path, target: &str, profile: &str) -> Result<(), BuildError> {
    let args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.to_string_lossy().to_string(),
        "--target".to_string(),
        target.to_string(),
        "--profile".to_string(),
        profile.to_string(),
    ];
    let output = run_command_capture("cargo", args.as_slice())?;
    if !output.status.success() {
        return Err(BuildError::CommandFailed {
            program: "cargo".to_string(),
            args,
            status_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

fn run_command_capture(program: &str, args: &[String]) -> Result<std::process::Output, BuildError> {
    let mut command = Command::new(program);
    command.env_remove("RUSTC_WRAPPER");
    for arg in args {
        command.arg(OsStr::new(arg));
    }
    let output = command
        .output()
        .map_err(|source| BuildError::Io { path: None, source })?;

    if !output.status.success() {
        return Err(BuildError::CommandFailed {
            program: program.to_string(),
            args: args.to_vec(),
            status_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(output)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn canonical_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn normalize_artifact_name(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_request() -> BuildRequest {
        BuildRequest {
            module_id: "test.module".to_string(),
            manifest_path: PathBuf::from("/tmp/module/Cargo.toml"),
            out_dir: PathBuf::from("/tmp/out"),
            target: DEFAULT_TARGET.to_string(),
            profile: DEFAULT_PROFILE.to_string(),
            dry_run: false,
        }
    }

    #[test]
    fn validate_request_rejects_empty_module_id() {
        let mut request = sample_request();
        request.module_id = "  ".to_string();
        let error = validate_request(&request).expect_err("expected empty module id to fail");
        assert!(matches!(error, BuildError::InvalidArgument(_)));
    }

    #[test]
    fn validate_request_rejects_invalid_profile() {
        let mut request = sample_request();
        request.profile = "staging".to_string();
        let error = validate_request(&request).expect_err("expected invalid profile to fail");
        assert!(matches!(error, BuildError::InvalidArgument(_)));
    }

    #[test]
    fn validate_request_rejects_empty_manifest_path() {
        let mut request = sample_request();
        request.manifest_path = PathBuf::new();
        let error = validate_request(&request).expect_err("expected empty manifest path to fail");
        assert!(matches!(error, BuildError::InvalidArgument(_)));
    }

    #[test]
    fn resolve_artifact_path_maps_profile_directory() {
        let metadata = CargoMetadata {
            packages: Vec::new(),
            target_directory: "/tmp/target".to_string(),
        };
        let release = resolve_artifact_path(&metadata, DEFAULT_TARGET, "release", "demo_module");
        let dev = resolve_artifact_path(&metadata, DEFAULT_TARGET, "dev", "demo_module");

        assert_eq!(
            release,
            Path::new("/tmp/target")
                .join(DEFAULT_TARGET)
                .join("release")
                .join("demo_module.wasm")
        );
        assert_eq!(
            dev,
            Path::new("/tmp/target")
                .join(DEFAULT_TARGET)
                .join("debug")
                .join("demo_module.wasm")
        );
    }

    #[test]
    fn find_wasm_target_prefers_cdylib_and_normalizes_name() {
        let package = CargoPackage {
            manifest_path: "/tmp/module/Cargo.toml".to_string(),
            targets: vec![
                CargoTarget {
                    name: "demo-lib".to_string(),
                    kind: vec!["lib".to_string()],
                },
                CargoTarget {
                    name: "demo-cdylib".to_string(),
                    kind: vec!["cdylib".to_string()],
                },
            ],
        };

        let target_name =
            find_wasm_target_name(&package).expect("expected cdylib target to be selected");
        assert_eq!(target_name, "demo_cdylib");
    }

    #[test]
    fn normalize_artifact_name_replaces_hyphen() {
        assert_eq!(normalize_artifact_name("alpha-beta"), "alpha_beta");
    }
}
