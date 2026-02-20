use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_WASM_TOOLCHAIN: &str = "nightly-2025-12-11";
const DEFAULT_WASM_TARGET: &str = "wasm32-unknown-unknown";
const DEFAULT_WASM_BUILD_STD: &str = "1";
const DEFAULT_WASM_BUILD_STD_COMPONENTS: &str = "std,panic_abort";
const DEFAULT_WASM_BUILD_STD_FEATURES: &str = "";
const DEFAULT_WASM_DETERMINISTIC_GUARD: &str = "1";
const DEFAULT_CANONICALIZER_VERSION: &str = "strip-custom-sections-v1";

#[derive(Debug)]
struct Options {
    module_ids_path: PathBuf,
    module_manifest_map_path: PathBuf,
    hash_manifest_path: PathBuf,
    identity_manifest_path: PathBuf,
    metadata_dir: PathBuf,
    workspace_root: PathBuf,
    profile: String,
    canonical_platforms_csv: String,
    check_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BuildRecipe {
    profile: String,
    wasm_toolchain: String,
    wasm_target: String,
    wasm_build_std: String,
    wasm_build_std_components: String,
    wasm_build_std_features: String,
    wasm_deterministic_guard: String,
    canonicalizer_version: String,
    canonical_platforms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ModuleIdentityEntry {
    module_id: String,
    hash_tokens: Vec<String>,
    source_hash: String,
    build_manifest_hash: String,
    identity_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IdentityManifest {
    schema_version: u32,
    module_set: String,
    build_recipe: BuildRecipe,
    modules: Vec<ModuleIdentityEntry>,
}

#[derive(Debug, Deserialize)]
struct BuildMetadata {
    module_id: String,
    wasm_hash_sha256: String,
}

fn usage() -> &'static str {
    "Usage: cargo run -p agent_world_distfs --bin sync_builtin_wasm_identity -- \\
  --module-ids-path <path> \\
  --module-manifest-map-path <path> \\
  --hash-manifest-path <path> \\
  --identity-manifest-path <path> \\
  --metadata-dir <dir> \\
  --workspace-root <dir> \\
  --profile <name> \\
  --canonical-platforms <csv> \\
  [--check]"
}

fn parse_args() -> Result<Options, String> {
    let mut module_ids_path: Option<PathBuf> = None;
    let mut module_manifest_map_path: Option<PathBuf> = None;
    let mut hash_manifest_path: Option<PathBuf> = None;
    let mut identity_manifest_path: Option<PathBuf> = None;
    let mut metadata_dir: Option<PathBuf> = None;
    let mut workspace_root: Option<PathBuf> = None;
    let mut profile: Option<String> = None;
    let mut canonical_platforms_csv: Option<String> = None;
    let mut check_only = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--module-ids-path" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --module-ids-path".to_string());
                };
                module_ids_path = Some(PathBuf::from(value));
            }
            "--module-manifest-map-path" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --module-manifest-map-path".to_string());
                };
                module_manifest_map_path = Some(PathBuf::from(value));
            }
            "--hash-manifest-path" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --hash-manifest-path".to_string());
                };
                hash_manifest_path = Some(PathBuf::from(value));
            }
            "--identity-manifest-path" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --identity-manifest-path".to_string());
                };
                identity_manifest_path = Some(PathBuf::from(value));
            }
            "--metadata-dir" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --metadata-dir".to_string());
                };
                metadata_dir = Some(PathBuf::from(value));
            }
            "--workspace-root" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --workspace-root".to_string());
                };
                workspace_root = Some(PathBuf::from(value));
            }
            "--profile" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --profile".to_string());
                };
                profile = Some(value);
            }
            "--canonical-platforms" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --canonical-platforms".to_string());
                };
                canonical_platforms_csv = Some(value);
            }
            "--check" => {
                check_only = true;
            }
            "-h" | "--help" => {
                return Err(usage().to_string());
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    let Some(module_ids_path) = module_ids_path else {
        return Err("--module-ids-path is required".to_string());
    };
    let Some(module_manifest_map_path) = module_manifest_map_path else {
        return Err("--module-manifest-map-path is required".to_string());
    };
    let Some(hash_manifest_path) = hash_manifest_path else {
        return Err("--hash-manifest-path is required".to_string());
    };
    let Some(identity_manifest_path) = identity_manifest_path else {
        return Err("--identity-manifest-path is required".to_string());
    };
    let Some(metadata_dir) = metadata_dir else {
        return Err("--metadata-dir is required".to_string());
    };
    let Some(workspace_root) = workspace_root else {
        return Err("--workspace-root is required".to_string());
    };
    let Some(profile) = profile else {
        return Err("--profile is required".to_string());
    };
    let Some(canonical_platforms_csv) = canonical_platforms_csv else {
        return Err("--canonical-platforms is required".to_string());
    };

    Ok(Options {
        module_ids_path,
        module_manifest_map_path,
        hash_manifest_path,
        identity_manifest_path,
        metadata_dir,
        workspace_root,
        profile,
        canonical_platforms_csv,
        check_only,
    })
}

fn read_module_ids(path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read module ids {}: {error}", path.display()))?;
    let ids: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();
    if ids.is_empty() {
        return Err(format!("module ids is empty: {}", path.display()));
    }
    Ok(ids)
}

fn read_module_manifest_map(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let content = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read module manifest map {}: {error}",
            path.display()
        )
    })?;

    let mut map = BTreeMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(module_id) = parts.next() else {
            continue;
        };
        let Some(manifest_path) = parts.next() else {
            return Err(format!("invalid module manifest map line: {line}"));
        };
        map.insert(module_id.to_string(), manifest_path.to_string());
    }

    Ok(map)
}

fn read_hash_manifest(path: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read hash manifest {}: {error}", path.display()))?;

    let mut map = BTreeMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(module_id) = parts.next() else {
            continue;
        };
        let tokens: Vec<String> = parts.map(ToString::to_string).collect();
        if tokens.is_empty() {
            return Err(format!(
                "hash manifest line has no tokens for module_id={module_id}"
            ));
        }
        map.insert(module_id.to_string(), tokens);
    }

    Ok(map)
}

fn parse_hash_value(token: &str) -> &str {
    token.split_once('=').map(|(_, hash)| hash).unwrap_or(token)
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn env_or_default(key: &str, default_value: &str) -> String {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_value.to_string())
}

fn parse_canonical_platforms(raw_csv: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut items = Vec::new();
    for raw in raw_csv.split(',') {
        let candidate = raw.trim();
        if candidate.is_empty() {
            continue;
        }
        if seen.insert(candidate.to_string()) {
            items.push(candidate.to_string());
        }
    }
    items
}

fn collect_source_files(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|error| format!("failed to read source dir {}: {error}", dir.display()))?;

    for entry in entries {
        let entry = entry
            .map_err(|error| format!("failed to read dir entry in {}: {error}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to read file type {}: {error}", path.display()))?;

        let rel = path.strip_prefix(root).map_err(|error| {
            format!("failed to strip source prefix {}: {error}", path.display())
        })?;

        if rel
            .components()
            .any(|component| component.as_os_str() == "target")
        {
            continue;
        }

        if file_type.is_dir() {
            collect_source_files(&path, root, out)?;
            continue;
        }

        if file_type.is_file() {
            out.push(path);
        }
    }

    Ok(())
}

fn compute_source_hash(
    module_dir: &Path,
    source_manifest_rel: &str,
    workspace_root: &Path,
) -> Result<String, String> {
    let mut files = Vec::new();
    collect_source_files(module_dir, module_dir, &mut files)?;

    files.sort_by(|left, right| {
        let left_rel = left.strip_prefix(module_dir).unwrap_or(left.as_path());
        let right_rel = right.strip_prefix(module_dir).unwrap_or(right.as_path());
        left_rel.to_string_lossy().cmp(&right_rel.to_string_lossy())
    });

    let mut hasher = Sha256::new();
    hasher.update(format!("source_manifest_rel={source_manifest_rel}\n").as_bytes());

    for file in files {
        let rel = file.strip_prefix(module_dir).map_err(|error| {
            format!(
                "failed to strip module dir prefix {}: {error}",
                file.display()
            )
        })?;
        let bytes = fs::read(&file)
            .map_err(|error| format!("failed to read source file {}: {error}", file.display()))?;
        let digest = sha256_hex(&bytes);
        hasher.update(format!("module_file:{}:{}\n", rel.to_string_lossy(), digest).as_bytes());
    }

    let lock_path = workspace_root.join("Cargo.lock");
    let lock_bytes = fs::read(&lock_path).map_err(|error| {
        format!(
            "failed to read workspace Cargo.lock {}: {error}",
            lock_path.display()
        )
    })?;
    hasher.update(format!("workspace_file:Cargo.lock:{}\n", sha256_hex(&lock_bytes)).as_bytes());

    Ok(format!("{:x}", hasher.finalize()))
}

fn module_set_from_ids(module_ids: &[String]) -> String {
    let mut set = BTreeSet::new();
    for module_id in module_ids {
        let prefix = module_id.split('.').next().unwrap_or(module_id.as_str());
        set.insert(prefix.to_string());
    }

    if set.len() == 1 {
        set.into_iter()
            .next()
            .unwrap_or_else(|| "builtin".to_string())
    } else {
        "mixed".to_string()
    }
}

fn expected_manifest(options: &Options) -> Result<IdentityManifest, String> {
    let module_ids = read_module_ids(&options.module_ids_path)?;
    let module_manifest_map = read_module_manifest_map(&options.module_manifest_map_path)?;
    let hash_manifest = read_hash_manifest(&options.hash_manifest_path)?;

    let canonical_platforms = parse_canonical_platforms(&options.canonical_platforms_csv);
    if canonical_platforms.is_empty() {
        return Err("canonical platforms list is empty".to_string());
    }

    let build_recipe = BuildRecipe {
        profile: options.profile.clone(),
        wasm_toolchain: env_or_default("AGENT_WORLD_WASM_TOOLCHAIN", DEFAULT_WASM_TOOLCHAIN),
        wasm_target: env_or_default("AGENT_WORLD_WASM_TARGET", DEFAULT_WASM_TARGET),
        wasm_build_std: env_or_default("AGENT_WORLD_WASM_BUILD_STD", DEFAULT_WASM_BUILD_STD),
        wasm_build_std_components: env_or_default(
            "AGENT_WORLD_WASM_BUILD_STD_COMPONENTS",
            DEFAULT_WASM_BUILD_STD_COMPONENTS,
        ),
        wasm_build_std_features: env_or_default(
            "AGENT_WORLD_WASM_BUILD_STD_FEATURES",
            DEFAULT_WASM_BUILD_STD_FEATURES,
        ),
        wasm_deterministic_guard: env_or_default(
            "AGENT_WORLD_WASM_DETERMINISTIC_GUARD",
            DEFAULT_WASM_DETERMINISTIC_GUARD,
        ),
        canonicalizer_version: DEFAULT_CANONICALIZER_VERSION.to_string(),
        canonical_platforms,
    };

    let build_manifest_hash = sha256_hex(
        serde_json::to_vec(&build_recipe)
            .map_err(|error| format!("failed to serialize build recipe: {error}"))?
            .as_slice(),
    );

    let mut modules = Vec::new();

    for module_id in &module_ids {
        let source_manifest_rel = module_manifest_map
            .get(module_id)
            .ok_or_else(|| format!("module manifest map missing module_id={module_id}"))?;

        let module_manifest_path = if Path::new(source_manifest_rel).is_absolute() {
            PathBuf::from(source_manifest_rel)
        } else {
            options.workspace_root.join(source_manifest_rel)
        };

        if !module_manifest_path.exists() {
            return Err(format!(
                "module manifest path not found module_id={} path={}",
                module_id,
                module_manifest_path.display()
            ));
        }

        let Some(module_dir) = module_manifest_path.parent() else {
            return Err(format!(
                "module manifest has no parent module_id={} path={}",
                module_id,
                module_manifest_path.display()
            ));
        };

        let hash_tokens = hash_manifest
            .get(module_id)
            .cloned()
            .ok_or_else(|| format!("hash manifest missing module_id={module_id}"))?;

        for token in &hash_tokens {
            let hash_value = parse_hash_value(token);
            if !is_sha256_hex(hash_value) {
                return Err(format!(
                    "invalid hash token module_id={} token={}",
                    module_id, token
                ));
            }
        }

        let metadata_path = options
            .metadata_dir
            .join(format!("{module_id}.metadata.json"));
        let metadata_content = fs::read(&metadata_path).map_err(|error| {
            format!(
                "failed to read build metadata module_id={} path={} err={error}",
                module_id,
                metadata_path.display()
            )
        })?;

        let metadata: BuildMetadata =
            serde_json::from_slice(&metadata_content).map_err(|error| {
                format!(
                    "failed to parse build metadata module_id={} path={} err={error}",
                    module_id,
                    metadata_path.display()
                )
            })?;

        if metadata.module_id != *module_id {
            return Err(format!(
                "build metadata module id mismatch expected={} actual={} path={}",
                module_id,
                metadata.module_id,
                metadata_path.display()
            ));
        }

        if !hash_tokens
            .iter()
            .map(|token| parse_hash_value(token))
            .any(|value| value == metadata.wasm_hash_sha256)
        {
            return Err(format!(
                "build metadata hash missing from hash manifest module_id={} built_hash={} hash_tokens=[{}]",
                module_id,
                metadata.wasm_hash_sha256,
                hash_tokens.join(",")
            ));
        }

        let source_hash =
            compute_source_hash(module_dir, source_manifest_rel, &options.workspace_root)?;
        let identity_hash =
            sha256_hex(format!("{module_id}:{source_hash}:{build_manifest_hash}").as_bytes());

        modules.push(ModuleIdentityEntry {
            module_id: module_id.clone(),
            hash_tokens,
            source_hash,
            build_manifest_hash: build_manifest_hash.clone(),
            identity_hash,
        });
    }

    Ok(IdentityManifest {
        schema_version: 1,
        module_set: module_set_from_ids(&module_ids),
        build_recipe,
        modules,
    })
}

fn sort_manifest(mut manifest: IdentityManifest) -> IdentityManifest {
    manifest
        .modules
        .sort_by(|left, right| left.module_id.cmp(&right.module_id));
    manifest
}

fn diff_manifest(expected: &IdentityManifest, actual: &IdentityManifest) -> String {
    if expected.schema_version != actual.schema_version {
        return format!(
            "schema_version mismatch expected={} actual={}",
            expected.schema_version, actual.schema_version
        );
    }

    if expected.module_set != actual.module_set {
        return format!(
            "module_set mismatch expected={} actual={}",
            expected.module_set, actual.module_set
        );
    }

    if expected.build_recipe != actual.build_recipe {
        return "build_recipe mismatch".to_string();
    }

    if expected.modules.len() != actual.modules.len() {
        return format!(
            "module entry count mismatch expected={} actual={}",
            expected.modules.len(),
            actual.modules.len()
        );
    }

    for (expected_entry, actual_entry) in expected.modules.iter().zip(actual.modules.iter()) {
        if expected_entry.module_id != actual_entry.module_id {
            return format!(
                "module_id mismatch expected={} actual={}",
                expected_entry.module_id, actual_entry.module_id
            );
        }
        if expected_entry.hash_tokens != actual_entry.hash_tokens {
            return format!(
                "hash_tokens mismatch module_id={} expected=[{}] actual=[{}]",
                expected_entry.module_id,
                expected_entry.hash_tokens.join(","),
                actual_entry.hash_tokens.join(",")
            );
        }
        if expected_entry.source_hash != actual_entry.source_hash {
            return format!(
                "source_hash mismatch module_id={} expected={} actual={}",
                expected_entry.module_id, expected_entry.source_hash, actual_entry.source_hash
            );
        }
        if expected_entry.build_manifest_hash != actual_entry.build_manifest_hash {
            return format!(
                "build_manifest_hash mismatch module_id={} expected={} actual={}",
                expected_entry.module_id,
                expected_entry.build_manifest_hash,
                actual_entry.build_manifest_hash
            );
        }
        if expected_entry.identity_hash != actual_entry.identity_hash {
            return format!(
                "identity_hash mismatch module_id={} expected={} actual={}",
                expected_entry.module_id, expected_entry.identity_hash, actual_entry.identity_hash
            );
        }
    }

    "unknown mismatch".to_string()
}

fn run() -> Result<(), String> {
    let options = parse_args()?;
    let expected = sort_manifest(expected_manifest(&options)?);

    if options.check_only {
        let manifest_bytes = fs::read(&options.identity_manifest_path).map_err(|error| {
            format!(
                "failed to read identity manifest {}: {error}",
                options.identity_manifest_path.display()
            )
        })?;
        let actual: IdentityManifest =
            serde_json::from_slice(&manifest_bytes).map_err(|error| {
                format!(
                    "failed to parse identity manifest {}: {error}",
                    options.identity_manifest_path.display()
                )
            })?;

        let actual = sort_manifest(actual);

        if expected != actual {
            return Err(format!(
                "identity manifest mismatch: {}",
                diff_manifest(&expected, &actual)
            ));
        }

        println!(
            "check ok: identity manifest is in sync module_count={} path={}",
            expected.modules.len(),
            options.identity_manifest_path.display()
        );
        return Ok(());
    }

    let payload = serde_json::to_vec_pretty(&expected)
        .map_err(|error| format!("failed to serialize identity manifest: {error}"))?;

    if let Some(parent) = options.identity_manifest_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create identity manifest dir {}: {error}",
                parent.display()
            )
        })?;
    }

    let mut with_newline = payload;
    with_newline.push(b'\n');
    fs::write(&options.identity_manifest_path, &with_newline).map_err(|error| {
        format!(
            "failed to write identity manifest {}: {error}",
            options.identity_manifest_path.display()
        )
    })?;

    println!(
        "synced builtin wasm identity manifest module_count={} path={}",
        expected.modules.len(),
        options.identity_manifest_path.display()
    );

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        eprintln!("{}", usage());
        std::process::exit(2);
    }
}
