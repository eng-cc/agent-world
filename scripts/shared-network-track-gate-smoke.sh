#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

run() {
  echo "+ $*"
  "$@"
}

ensure_file_contains() {
  local file=$1
  local pattern=$2
  if ! rg -q -- "$pattern" "$file"; then
    echo "error: pattern not found: $pattern" >&2
    echo "  file=$file" >&2
    exit 1
  fi
}

smoke_root=".tmp/shared_network_track_gate_smoke"
rm -rf "$smoke_root"
mkdir -p "$smoke_root/runtime" "$smoke_root/world" "$smoke_root/evidence"

printf 'runtime-build-v1\n' >"$smoke_root/runtime/runtime.bin"
printf 'snapshot\n' >"$smoke_root/world/state.txt"
printf '{"signers":["signer01"]}\n' >"$smoke_root/world/public_manifest.json"
printf '# evidence\n' >"$smoke_root/evidence/shared-access.md"
printf '# evidence\n' >"$smoke_root/evidence/multi-entry.md"
printf '# evidence\n' >"$smoke_root/evidence/governance-drill.md"
printf '# evidence\n' >"$smoke_root/evidence/longrun.md"
printf '# evidence\n' >"$smoke_root/evidence/rollback.md"

bundle_path="$smoke_root/candidate.json"
run ./scripts/release-candidate-bundle.sh create \
  --bundle "$bundle_path" \
  --candidate-id "shared-devnet-gate-smoke-01" \
  --track "shared_devnet" \
  --runtime-build-ref "$smoke_root/runtime/runtime.bin" \
  --world-snapshot-ref "$smoke_root/world" \
  --governance-manifest-ref "$smoke_root/world/public_manifest.json" \
  --evidence-ref "$smoke_root/evidence/shared-access.md" \
  --allow-dirty-worktree

lanes_pass="$smoke_root/lanes-pass.tsv"
cat >"$lanes_pass" <<EOF
candidate_bundle_integrity	qa_engineer	pass	$smoke_root/evidence/shared-access.md	bundle validates
shared_access	qa_engineer	pass	$smoke_root/evidence/shared-access.md	shared endpoint reachable
multi_entry_closure	qa_engineer	pass	$smoke_root/evidence/multi-entry.md	web + api + no-ui aligned
governance_live_drill	runtime_engineer	pass	$smoke_root/evidence/governance-drill.md	live drill complete
short_window_longrun	runtime_engineer	pass	$smoke_root/evidence/longrun.md	short window stable
rollback_target_ready	liveops_community	pass	$smoke_root/evidence/rollback.md	fallback candidate recorded
EOF

pass_out="$smoke_root/pass"
run ./scripts/shared-network-track-gate.sh \
  --track shared_devnet \
  --candidate-bundle "$bundle_path" \
  --lanes-tsv "$lanes_pass" \
  --out-dir "$pass_out"

pass_summary=$(ls -t "$pass_out"/*/summary.json | head -n 1)
ensure_file_contains "$pass_summary" '"gate_result": "pass"'
ensure_file_contains "$pass_summary" '"promotion_recommendation": "eligible_for_promotion"'

lanes_partial="$smoke_root/lanes-partial.tsv"
cat >"$lanes_partial" <<EOF
candidate_bundle_integrity	qa_engineer	pass	$smoke_root/evidence/shared-access.md	bundle validates
shared_access	qa_engineer	pass	$smoke_root/evidence/shared-access.md	shared endpoint reachable
multi_entry_closure	qa_engineer	partial	$smoke_root/evidence/multi-entry.md	no-ui lane pending
governance_live_drill	runtime_engineer	pass	$smoke_root/evidence/governance-drill.md	live drill complete
short_window_longrun	runtime_engineer	pass	$smoke_root/evidence/longrun.md	short window stable
rollback_target_ready	liveops_community	pass	$smoke_root/evidence/rollback.md	fallback candidate recorded
EOF

partial_out="$smoke_root/partial"
run ./scripts/shared-network-track-gate.sh \
  --track shared_devnet \
  --candidate-bundle "$bundle_path" \
  --lanes-tsv "$lanes_partial" \
  --out-dir "$partial_out"

partial_summary=$(ls -t "$partial_out"/*/summary.json | head -n 1)
ensure_file_contains "$partial_summary" '"gate_result": "partial"'
ensure_file_contains "$partial_summary" '"promotion_recommendation": "hold_promotion"'

lanes_block="$smoke_root/lanes-block.tsv"
cat >"$lanes_block" <<EOF
candidate_bundle_integrity	qa_engineer	pass	$smoke_root/evidence/shared-access.md	bundle validates
shared_access	qa_engineer	pass	$smoke_root/evidence/shared-access.md	shared endpoint reachable
governance_live_drill	runtime_engineer	pass	$smoke_root/evidence/governance-drill.md	live drill complete
short_window_longrun	runtime_engineer	pass	$smoke_root/evidence/longrun.md	short window stable
rollback_target_ready	liveops_community	pass	$smoke_root/evidence/rollback.md	fallback candidate recorded
EOF

block_out="$smoke_root/block"
run ./scripts/shared-network-track-gate.sh \
  --track shared_devnet \
  --candidate-bundle "$bundle_path" \
  --lanes-tsv "$lanes_block" \
  --out-dir "$block_out"

block_summary=$(ls -t "$block_out"/*/summary.json | head -n 1)
ensure_file_contains "$block_summary" '"gate_result": "block"'
ensure_file_contains "$block_summary" '"missing_required_lanes": \['
ensure_file_contains "$block_summary" '"multi_entry_closure"'

echo "shared network track gate smoke checks passed"
