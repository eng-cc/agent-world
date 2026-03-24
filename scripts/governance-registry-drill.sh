#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/governance-registry-drill.sh \
    --source-world-dir <dir> \
    --baseline-manifest <public_manifest.json> \
    --slot-id <slot_id> \
    --replace-signer-id <signer_id> \
    [--replacement-signer-id <signer_id>] \
    --replacement-public-key <hex> \
    --out-dir <dir>

Description:
  Runs a clone-world governance registry drill with two cases:
  1. pass case: rotate one signer inside a slot while preserving 2-of-3
  2. block case: intentionally degrade one slot to 2-of-2
  Note:
  - controller slots may keep the same signer_id and replace only the public key
  - finality slot rotation must use a new signer_id via --replacement-signer-id

Artifacts:
  <out-dir>/run_config.json
  <out-dir>/manifests/{rotated_pass_manifest.json,degraded_block_manifest.json}
  <out-dir>/logs/*
  <out-dir>/summary.json
  <out-dir>/summary.md
EOF
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

slugify() {
  printf '%s' "$1" | tr -cs '[:alnum:]._-' '-'
}

run_and_capture() {
  local name="$1"
  shift
  local stdout_path="$LOG_DIR/${name}.stdout"
  local stderr_path="$LOG_DIR/${name}.stderr"
  local rc_path="$LOG_DIR/${name}.rc"
  local rc=0
  if "$@" >"$stdout_path" 2>"$stderr_path"; then
    rc=0
  else
    rc=$?
  fi
  printf '%s\n' "$rc" >"$rc_path"
  return 0
}

SOURCE_WORLD_DIR=""
BASELINE_MANIFEST=""
SLOT_ID=""
REPLACE_SIGNER_ID=""
REPLACEMENT_SIGNER_ID=""
REPLACEMENT_PUBLIC_KEY=""
OUT_DIR=""
FINALITY_SLOT_ID="governance.finality.v1"
EXPECTED_THRESHOLD="2"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source-world-dir)
      SOURCE_WORLD_DIR="$2"
      shift 2
      ;;
    --baseline-manifest)
      BASELINE_MANIFEST="$2"
      shift 2
      ;;
    --slot-id)
      SLOT_ID="$2"
      shift 2
      ;;
    --replace-signer-id)
      REPLACE_SIGNER_ID="$2"
      shift 2
      ;;
    --replacement-signer-id)
      REPLACEMENT_SIGNER_ID="$2"
      shift 2
      ;;
    --replacement-public-key)
      REPLACEMENT_PUBLIC_KEY="$2"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$SOURCE_WORLD_DIR" || -z "$BASELINE_MANIFEST" || -z "$SLOT_ID" || -z "$REPLACE_SIGNER_ID" || -z "$REPLACEMENT_PUBLIC_KEY" || -z "$OUT_DIR" ]]; then
  echo "all flags are required" >&2
  usage
  exit 1
fi

if [[ -z "$REPLACEMENT_SIGNER_ID" ]]; then
  REPLACEMENT_SIGNER_ID="$REPLACE_SIGNER_ID"
fi

if [[ "$SLOT_ID" == "$FINALITY_SLOT_ID" && "$REPLACEMENT_SIGNER_ID" == "$REPLACE_SIGNER_ID" ]]; then
  echo "finality slot rotation requires a new signer id; pass --replacement-signer-id for $SLOT_ID" >&2
  exit 1
fi

require_command jq
require_command cp
require_command date

if [[ ! -d "$SOURCE_WORLD_DIR" ]]; then
  echo "source world dir does not exist: $SOURCE_WORLD_DIR" >&2
  exit 1
fi
if [[ ! -f "$BASELINE_MANIFEST" ]]; then
  echo "baseline manifest does not exist: $BASELINE_MANIFEST" >&2
  exit 1
fi
if [[ ! "$REPLACEMENT_PUBLIC_KEY" =~ ^[0-9a-fA-F]{64}$ ]]; then
  echo "replacement public key must be 32-byte hex" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
MANIFEST_DIR="$OUT_DIR/manifests"
LOG_DIR="$OUT_DIR/logs"
PASS_WORLD_DIR="$OUT_DIR/pass-world"
BLOCK_WORLD_DIR="$OUT_DIR/block-world"
mkdir -p "$MANIFEST_DIR" "$LOG_DIR"
rm -rf "$PASS_WORLD_DIR" "$BLOCK_WORLD_DIR"

PASS_MANIFEST="$MANIFEST_DIR/rotated_pass_manifest.json"
BLOCK_MANIFEST="$MANIFEST_DIR/degraded_block_manifest.json"

BASELINE_SLOT_COUNT="$(jq --arg slot "$SLOT_ID" '[.[] | select(.slot_id == $slot)] | length' "$BASELINE_MANIFEST")"
MATCHING_SIGNER_COUNT="$(jq --arg slot "$SLOT_ID" --arg signer "$REPLACE_SIGNER_ID" '[.[] | select(.slot_id == $slot and .signer_id == $signer)] | length' "$BASELINE_MANIFEST")"
REPLACEMENT_SIGNER_EXISTS_COUNT="$(jq --arg slot "$SLOT_ID" --arg signer "$REPLACEMENT_SIGNER_ID" '[.[] | select(.slot_id == $slot and .signer_id == $signer)] | length' "$BASELINE_MANIFEST")"
if [[ "$BASELINE_SLOT_COUNT" != "3" ]]; then
  echo "expected exactly 3 manifest entries for slot $SLOT_ID, got $BASELINE_SLOT_COUNT" >&2
  exit 1
fi
if [[ "$MATCHING_SIGNER_COUNT" != "1" ]]; then
  echo "expected exactly 1 manifest entry for slot $SLOT_ID signer $REPLACE_SIGNER_ID, got $MATCHING_SIGNER_COUNT" >&2
  exit 1
fi
if [[ "$REPLACEMENT_SIGNER_ID" != "$REPLACE_SIGNER_ID" && "$REPLACEMENT_SIGNER_EXISTS_COUNT" != "0" ]]; then
  echo "replacement signer id already exists in slot $SLOT_ID: $REPLACEMENT_SIGNER_ID" >&2
  exit 1
fi

jq \
  --arg slot "$SLOT_ID" \
  --arg signer "$REPLACE_SIGNER_ID" \
  --arg replacement_signer "$REPLACEMENT_SIGNER_ID" \
  --arg replacement_public_key "$REPLACEMENT_PUBLIC_KEY" \
  '
  map(
    if .slot_id == $slot and .signer_id == $signer then
      .signer_id = $replacement_signer
      | .public_key_hex = $replacement_public_key
      | .awt_account_id = ("awt:pk:" + $replacement_public_key)
    else
      .
    end
  )
  ' \
  "$BASELINE_MANIFEST" >"$PASS_MANIFEST"

jq \
  --arg slot "$SLOT_ID" \
  --arg signer "$REPLACE_SIGNER_ID" \
  '
  map(select(.slot_id != $slot or .signer_id != $signer))
  ' \
  "$BASELINE_MANIFEST" >"$BLOCK_MANIFEST"

PASS_SLOT_COUNT="$(jq --arg slot "$SLOT_ID" '[.[] | select(.slot_id == $slot)] | length' "$PASS_MANIFEST")"
BLOCK_SLOT_COUNT="$(jq --arg slot "$SLOT_ID" '[.[] | select(.slot_id == $slot)] | length' "$BLOCK_MANIFEST")"
if [[ "$PASS_SLOT_COUNT" != "3" ]]; then
  echo "pass manifest must keep 3 entries for slot $SLOT_ID, got $PASS_SLOT_COUNT" >&2
  exit 1
fi
if [[ "$BLOCK_SLOT_COUNT" != "2" ]]; then
  echo "block manifest must degrade slot $SLOT_ID to 2 entries, got $BLOCK_SLOT_COUNT" >&2
  exit 1
fi

cp -a "$SOURCE_WORLD_DIR" "$PASS_WORLD_DIR"
cp -a "$SOURCE_WORLD_DIR" "$BLOCK_WORLD_DIR"

run_and_capture baseline_audit \
  env -u RUSTC_WRAPPER cargo run -p oasis7 --bin oasis7_governance_registry_audit -- \
    --world-dir "$SOURCE_WORLD_DIR" \
    --public-manifest "$BASELINE_MANIFEST" \
    --finality-slot-id "$FINALITY_SLOT_ID" \
    --expected-threshold "$EXPECTED_THRESHOLD" \
    --strict-manifest-match \
    --require-single-failure-tolerance

run_and_capture pass_import \
  env -u RUSTC_WRAPPER cargo run -p oasis7 --bin oasis7_governance_registry_import -- \
    --world-dir "$PASS_WORLD_DIR" \
    --public-manifest "$PASS_MANIFEST"

run_and_capture pass_audit \
  env -u RUSTC_WRAPPER cargo run -p oasis7 --bin oasis7_governance_registry_audit -- \
    --world-dir "$PASS_WORLD_DIR" \
    --public-manifest "$PASS_MANIFEST" \
    --finality-slot-id "$FINALITY_SLOT_ID" \
    --expected-threshold "$EXPECTED_THRESHOLD" \
    --strict-manifest-match \
    --require-single-failure-tolerance

run_and_capture block_import \
  env -u RUSTC_WRAPPER cargo run -p oasis7 --bin oasis7_governance_registry_import -- \
    --world-dir "$BLOCK_WORLD_DIR" \
    --public-manifest "$BLOCK_MANIFEST"

run_and_capture block_audit \
  env -u RUSTC_WRAPPER cargo run -p oasis7 --bin oasis7_governance_registry_audit -- \
    --world-dir "$BLOCK_WORLD_DIR" \
    --public-manifest "$BLOCK_MANIFEST" \
    --finality-slot-id "$FINALITY_SLOT_ID" \
    --expected-threshold "$EXPECTED_THRESHOLD" \
    --strict-manifest-match \
    --require-single-failure-tolerance

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
PASS_AUDIT_RC="$(cat "$LOG_DIR/pass_audit.rc")"
BLOCK_AUDIT_RC="$(cat "$LOG_DIR/block_audit.rc")"
BASELINE_AUDIT_RC="$(cat "$LOG_DIR/baseline_audit.rc")"
PASS_IMPORT_RC="$(cat "$LOG_DIR/pass_import.rc")"
BLOCK_IMPORT_RC="$(cat "$LOG_DIR/block_import.rc")"

jq -n \
  --arg generated_at_utc "$TIMESTAMP" \
  --arg source_world_dir "$SOURCE_WORLD_DIR" \
  --arg baseline_manifest "$BASELINE_MANIFEST" \
  --arg slot_id "$SLOT_ID" \
  --arg replace_signer_id "$REPLACE_SIGNER_ID" \
  --arg replacement_signer_id "$REPLACEMENT_SIGNER_ID" \
  --arg replacement_public_key "$REPLACEMENT_PUBLIC_KEY" \
  --arg pass_world_dir "$PASS_WORLD_DIR" \
  --arg block_world_dir "$BLOCK_WORLD_DIR" \
  --arg pass_manifest "$PASS_MANIFEST" \
  --arg block_manifest "$BLOCK_MANIFEST" \
  --argjson baseline_audit_rc "$BASELINE_AUDIT_RC" \
  --argjson pass_import_rc "$PASS_IMPORT_RC" \
  --argjson pass_audit_rc "$PASS_AUDIT_RC" \
  --argjson block_import_rc "$BLOCK_IMPORT_RC" \
  --argjson block_audit_rc "$BLOCK_AUDIT_RC" \
  --slurpfile baseline_audit_json "$LOG_DIR/baseline_audit.stdout" \
  --slurpfile pass_import_json "$LOG_DIR/pass_import.stdout" \
  --slurpfile pass_audit_json "$LOG_DIR/pass_audit.stdout" \
  --slurpfile block_import_json "$LOG_DIR/block_import.stdout" \
  --slurpfile block_audit_json "$LOG_DIR/block_audit.stdout" \
  '
  {
    generated_at_utc: $generated_at_utc,
    source_world_dir: $source_world_dir,
    baseline_manifest: $baseline_manifest,
    slot_id: $slot_id,
    replace_signer_id: $replace_signer_id,
    replacement_signer_id: $replacement_signer_id,
    replacement_public_key: $replacement_public_key,
    baseline: {
      audit_rc: $baseline_audit_rc,
      audit_report: $baseline_audit_json[0],
      expectation_met: ($baseline_audit_rc == 0 and $baseline_audit_json[0].overall_status == "ready_for_ops_drill")
    },
    pass_case: {
      world_dir: $pass_world_dir,
      manifest: $pass_manifest,
      import_rc: $pass_import_rc,
      import_summary: $pass_import_json[0],
      audit_rc: $pass_audit_rc,
      audit_report: $pass_audit_json[0],
      expectation_met: ($pass_import_rc == 0 and $pass_audit_rc == 0 and $pass_audit_json[0].overall_status == "ready_for_ops_drill")
    },
    block_case: {
      world_dir: $block_world_dir,
      manifest: $block_manifest,
      import_rc: $block_import_rc,
      import_summary: $block_import_json[0],
      audit_rc: $block_audit_rc,
      audit_report: $block_audit_json[0],
      expectation_met: ($block_import_rc == 0 and $block_audit_rc == 2 and $block_audit_json[0].overall_status == "failover_blocked")
    }
  }
  ' >"$OUT_DIR/summary.json"

BASELINE_STATUS="$(jq -r '.baseline.audit_report.overall_status' "$OUT_DIR/summary.json")"
PASS_STATUS="$(jq -r '.pass_case.audit_report.overall_status' "$OUT_DIR/summary.json")"
BLOCK_STATUS="$(jq -r '.block_case.audit_report.overall_status' "$OUT_DIR/summary.json")"

cat >"$OUT_DIR/run_config.json" <<EOF
{
  "source_world_dir": "$SOURCE_WORLD_DIR",
  "baseline_manifest": "$BASELINE_MANIFEST",
  "slot_id": "$SLOT_ID",
  "replace_signer_id": "$REPLACE_SIGNER_ID",
  "replacement_signer_id": "$REPLACEMENT_SIGNER_ID",
  "replacement_public_key": "$REPLACEMENT_PUBLIC_KEY",
  "out_dir": "$OUT_DIR"
}
EOF

cat >"$OUT_DIR/summary.md" <<EOF
# Governance Registry Drill Summary

- generated_at_utc: $TIMESTAMP
- slot_id: \`$SLOT_ID\`
- replace_signer_id: \`$REPLACE_SIGNER_ID\`
- replacement_signer_id: \`$REPLACEMENT_SIGNER_ID\`
- replacement_public_key: \`$REPLACEMENT_PUBLIC_KEY\`

## Baseline
- audit_rc: \`$BASELINE_AUDIT_RC\`
- overall_status: \`$BASELINE_STATUS\`

## Pass Case
- manifest: \`$PASS_MANIFEST\`
- world_dir: \`$PASS_WORLD_DIR\`
- import_rc: \`$PASS_IMPORT_RC\`
- audit_rc: \`$PASS_AUDIT_RC\`
- overall_status: \`$PASS_STATUS\`

## Block Case
- manifest: \`$BLOCK_MANIFEST\`
- world_dir: \`$BLOCK_WORLD_DIR\`
- import_rc: \`$BLOCK_IMPORT_RC\`
- audit_rc: \`$BLOCK_AUDIT_RC\`
- overall_status: \`$BLOCK_STATUS\`

## Logs
- logs dir: \`$LOG_DIR\`
- summary json: \`$OUT_DIR/summary.json\`
EOF

printf 'governance registry drill completed: %s\n' "$OUT_DIR"
