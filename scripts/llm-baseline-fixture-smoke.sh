#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

fixture_dir="fixtures/llm_baseline/state_01"
snapshot_path="$fixture_dir/snapshot.json"
journal_path="$fixture_dir/journal.json"

if [[ ! -f "$snapshot_path" ]]; then
  echo "missing baseline snapshot fixture: $snapshot_path" >&2
  exit 2
fi

if [[ ! -f "$journal_path" ]]; then
  echo "missing baseline journal fixture: $journal_path" >&2
  exit 2
fi

echo "+ baseline fixture: $fixture_dir"
echo "+ env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full simulator::tests::persist::kernel_loads_tracked_llm_baseline_fixture_state -- --nocapture"
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full simulator::tests::persist::kernel_loads_tracked_llm_baseline_fixture_state -- --nocapture
echo "+ env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_llm_agent_demo --features test_tier_full runtime_bridge_continues_governance_from_tracked_baseline_fixture -- --nocapture"
env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_llm_agent_demo --features test_tier_full runtime_bridge_continues_governance_from_tracked_baseline_fixture -- --nocapture
echo "+ env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_llm_agent_demo --features test_tier_full runtime_bridge_civic_hotspot_preset_seeds_followup_handles -- --nocapture"
env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_llm_agent_demo --features test_tier_full runtime_bridge_civic_hotspot_preset_seeds_followup_handles -- --nocapture
