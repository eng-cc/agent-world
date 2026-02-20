#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

metadata_dir="$repo_root/.tmp/builtin-wasm-sync-modules"
hash_manifest_path="$repo_root/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256"
identity_manifest_path="$repo_root/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.identity.json"
runner_label=""
out_path=""

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci-m1-wasm-summary.sh --out <path> [--runner-label <label>] [--metadata-dir <dir>]

Options:
  --out <path>            Output summary JSON path (required)
  --runner-label <label>  Runner label recorded in summary (default: detected platform key)
  --metadata-dir <dir>    Metadata dir produced by sync script
                          (default: .tmp/builtin-wasm-sync-modules)
  -h, --help              Show this help
USAGE
}

normalize_platform_os() {
  local raw="$1"
  case "$raw" in
    Darwin) echo "darwin" ;;
    Linux) echo "linux" ;;
    *) echo "$raw" | tr '[:upper:]' '[:lower:]' ;;
  esac
}

normalize_platform_arch() {
  local raw="$1"
  case "$raw" in
    arm64|aarch64) echo "arm64" ;;
    x86_64|amd64) echo "x86_64" ;;
    *) echo "$raw" ;;
  esac
}

detect_current_platform() {
  local os arch
  os="$(normalize_platform_os "$(uname -s)")"
  arch="$(normalize_platform_arch "$(uname -m)")"
  echo "${os}-${arch}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      [[ $# -ge 2 ]] || { echo "error: --out requires a value" >&2; exit 2; }
      out_path="$2"
      shift 2
      ;;
    --runner-label)
      [[ $# -ge 2 ]] || { echo "error: --runner-label requires a value" >&2; exit 2; }
      runner_label="$2"
      shift 2
      ;;
    --metadata-dir)
      [[ $# -ge 2 ]] || { echo "error: --metadata-dir requires a value" >&2; exit 2; }
      metadata_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$out_path" ]]; then
  echo "error: --out is required" >&2
  usage
  exit 2
fi

current_platform="$(detect_current_platform)"
if [[ -z "$runner_label" ]]; then
  runner_label="$current_platform"
fi

./scripts/sync-m1-builtin-wasm-artifacts.sh --check

mkdir -p "$(dirname "$out_path")"

python3 - "$metadata_dir" "$hash_manifest_path" "$identity_manifest_path" "$out_path" "$runner_label" "$current_platform" <<'PY'
import datetime as dt
import glob
import json
import pathlib
import sys

metadata_dir = pathlib.Path(sys.argv[1])
hash_manifest_path = pathlib.Path(sys.argv[2])
identity_manifest_path = pathlib.Path(sys.argv[3])
out_path = pathlib.Path(sys.argv[4])
runner_label = sys.argv[5]
current_platform = sys.argv[6]

manifest_platform_hashes = {}
canonical_platforms = set()
legacy_mode = False
for lineno, raw_line in enumerate(hash_manifest_path.read_text().splitlines(), start=1):
    line = raw_line.strip()
    if not line:
        continue
    tokens = line.split()
    if len(tokens) < 2:
        raise SystemExit(
            f"error: invalid hash manifest line {lineno} in {hash_manifest_path}: {raw_line}"
        )
    module_id = tokens[0]
    keyed_tokens = [token for token in tokens[1:] if "=" in token]
    if keyed_tokens:
        selected_hash = None
        for token in keyed_tokens:
            platform_key, hash_value = token.split("=", 1)
            canonical_platforms.add(platform_key)
            if platform_key == current_platform:
                selected_hash = hash_value
        if selected_hash is None:
            raise SystemExit(
                f"error: hash manifest missing platform {current_platform} for module {module_id}"
            )
        manifest_platform_hashes[module_id] = selected_hash
    else:
        legacy_mode = True
        manifest_platform_hashes[module_id] = tokens[1]

manifest_module_ids = set(manifest_platform_hashes.keys())

metadata_files = sorted(glob.glob(str(metadata_dir / "*.metadata.json")))
if not metadata_files:
    raise SystemExit(f"error: no metadata files found in {metadata_dir}")

module_hashes = {}
for path in metadata_files:
    payload = json.loads(pathlib.Path(path).read_text())
    module_id = payload.get("module_id")
    wasm_hash = payload.get("wasm_hash_sha256")
    if not module_id or not wasm_hash:
        raise SystemExit(f"error: invalid metadata payload in {path}")
    if module_id not in manifest_module_ids:
        continue
    module_hashes[module_id] = wasm_hash

identity_payload = json.loads(identity_manifest_path.read_text())
identity_modules = identity_payload.get("modules", [])
if not isinstance(identity_modules, list):
    raise SystemExit(f"error: invalid identity manifest modules field in {identity_manifest_path}")

identity_hashes = {}
for entry in identity_modules:
    module_id = entry.get("module_id")
    identity_hash = entry.get("identity_hash")
    if not module_id or not identity_hash:
        raise SystemExit(
            f"error: identity manifest entry missing module_id/identity_hash in {identity_manifest_path}"
        )
    if module_id not in manifest_module_ids:
        continue
    identity_hashes[module_id] = identity_hash

module_id_set = set(module_hashes.keys())
if module_id_set != manifest_module_ids:
    missing = sorted(module_id_set - manifest_module_ids)
    extra = sorted(manifest_module_ids - module_id_set)
    raise SystemExit(
        f"error: module set mismatch between metadata and hash manifest missing={missing} extra={extra}"
    )
if module_id_set != set(identity_hashes.keys()):
    missing = sorted(module_id_set - set(identity_hashes.keys()))
    extra = sorted(set(identity_hashes.keys()) - module_id_set)
    raise SystemExit(
        f"error: module set mismatch between metadata and identity manifest missing={missing} extra={extra}"
    )

for module_id, built_hash in sorted(module_hashes.items()):
    expected_hash = manifest_platform_hashes[module_id]
    if built_hash != expected_hash:
        raise SystemExit(
            f"error: built hash does not match manifest for module {module_id} built={built_hash} expected={expected_hash}"
        )

summary = {
    "schema_version": 1,
    "runner": runner_label,
    "current_platform": current_platform,
    "generated_at_utc": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
    "module_count": len(module_hashes),
    "module_hashes": dict(sorted(module_hashes.items())),
    "manifest_platform_hashes": dict(sorted(manifest_platform_hashes.items())),
    "identity_hashes": dict(sorted(identity_hashes.items())),
    "canonical_platforms": sorted(canonical_platforms),
    "legacy_manifest_mode": legacy_mode,
    "hash_manifest_path": str(hash_manifest_path),
    "identity_manifest_path": str(identity_manifest_path),
}

out_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
print(f"wrote m1 wasm summary: {out_path}")
PY
