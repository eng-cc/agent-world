# Shared Network Follow-up `shared_devnet` Window Evidence (2026-03-24)

审计轮次: 1

## Meta
- 关联专题:
  - `PRD-P2P-RTMIN-002`
  - `PRD-P2P-RTMIN-003`
  - `PRD-P2P-BENCH-003`
- 关联任务:
  - `RTMIN-4A`
- 责任角色:
  - `qa_engineer`
- 协作角色:
  - `runtime_engineer`
  - `liveops_community`
- 当前结论:
  - `partial`
- 目标:
  - 用 fresh candidate 复跑 same-candidate `headed Web + no-ui + pure_api`，确认 `multi_entry_closure` 不再因为 stale bundle 或端口冲突被阻断。

## 执行范围
- `window_id`:
  - `shared-devnet-20260324-05`
- `candidate_id`:
  - `shared-devnet-20260324-05`
- git commit:
  - `50257fca76de317a605ff51d1dba3b584deb3759`
- candidate bundle:
  - `output/release-candidates/shared-devnet-20260324-05.json`
- runtime bundle:
  - `output/release/game-launcher-shared-devnet-20260324-02/`
- world snapshot:
  - `output/chain-runtime/viewer-live-node/reward-runtime-execution-world`
- governance manifest:
  - `output/governance-drills/20260324-finality-live-world-signer04/manifests/rotated_pass_manifest.json`
- rehearsal root:
  - `output/shared-network/shared-devnet-20260324-05/`

## 执行命令
- bundle rebuild:
  - `./scripts/build-game-launcher-bundle.sh --out-dir output/release/game-launcher-shared-devnet-20260324-02`
- shared-devnet window:
  - `./scripts/shared-devnet-rehearsal.sh --window-id shared-devnet-20260324-05 --candidate-id shared-devnet-20260324-05 --candidate-bundle-out output/release-candidates/shared-devnet-20260324-05.json --runtime-build-ref output/release/game-launcher-shared-devnet-20260324-02/bin/oasis7_chain_runtime --world-snapshot-ref output/chain-runtime/viewer-live-node/reward-runtime-execution-world --governance-manifest-ref output/governance-drills/20260324-finality-live-world-signer04/manifests/rotated_pass_manifest.json --evidence-ref doc/testing/evidence/governance-registry-live-world-drill-finality-2026-03-24.md --evidence-ref doc/testing/evidence/governance-registry-live-world-drill-foundation-ops-2026-03-24.md --note 'fresh shared-devnet candidate after lane port isolation fix' --bundle-dir output/release/game-launcher-shared-devnet-20260324-02 --viewer-port 4190 --live-bind 127.0.0.1:5140 --web-bind 127.0.0.1:5130 --release-gate-mode dry-run --web-mode execute --headless-mode execute --pure-api-mode execute --governance-mode evidence --governance-window-evidence-ref doc/testing/evidence/governance-registry-live-world-drill-finality-2026-03-24.md --longrun-mode dry-run`

## 关键产物
- candidate bundle:
  - `output/release-candidates/shared-devnet-20260324-05.json`
- release-gate dry-run:
  - `output/shared-network/shared-devnet-20260324-05/release-gate/20260324-171227/release-gate-summary.md`
- multi-entry evidence:
  - `output/shared-network/shared-devnet-20260324-05/multi-entry/web/post-onboarding-20260324-171230/post-onboarding-summary.md`
  - `output/shared-network/shared-devnet-20260324-05/multi-entry/headless/post-onboarding-headless-20260324-171243/post-onboarding-headless-summary.md`
  - `output/shared-network/shared-devnet-20260324-05/multi-entry/pure-api/pure-api-required-20260324-171245/pure-api-summary.md`
  - `output/shared-network/shared-devnet-20260324-05/multi-entry-summary.md`
- lane scaffold:
  - `output/shared-network/shared-devnet-20260324-05/access-check.md`
  - `output/shared-network/shared-devnet-20260324-05/governance-summary.md`
  - `output/shared-network/shared-devnet-20260324-05/longrun-summary.md`
  - `output/shared-network/shared-devnet-20260324-05/rollback-target.md`
  - `output/shared-network/shared-devnet-20260324-05/lanes.shared_devnet.tsv`
- shared-network gate:
  - `output/shared-network/shared-devnet-20260324-05/gate/shared_devnet-20260324-171248/summary.md`
  - `output/shared-network/shared-devnet-20260324-05/gate/shared_devnet-20260324-171248/summary.json`
- liveops records:
  - `doc/testing/evidence/shared-network-shared-devnet-follow-up-promotion-record-2026-03-24.md`
  - `doc/testing/evidence/shared-network-shared-devnet-follow-up-incident-2026-03-24.md`

## QA 结果
- `release_candidate_bundle`:
  - `validation=ok`
  - `git_worktree_dirty=false`
  - git commit 已更新到端口隔离修正后的 `50257fca`
- `multi_entry_closure`:
  - `headed Web=pass`
  - `no-ui=pass`
  - `pure_api=pass`
  - lane verdict `pass`
- `governance_live_drill`:
  - `pass`
  - 本轮仍是 same-window evidence reuse，而非新 live drill
- `shared-network-track-gate`:
  - `gate_result=partial`
  - `promotion_recommendation=hold_promotion`
- remaining partial lanes:
  - `shared_access`
  - `short_window_longrun`
  - `rollback_target_ready`

## 结论
- 本轮已经把 `shared_devnet` 的 `multi_entry_closure` 从“未重跑/被 blocker 卡死”推进到正式 `pass`。
- 这证明当前 shared-devnet orchestration 已可稳定收同一 candidate 的三条入口证据，且端口隔离修正有效。
- 但整体 shared-devnet 仍未到 `pass`，因为还缺：
  - 真实 shared access
  - 非 dry-run 的 short-window S9/S10 结果
  - 可追溯的 fallback candidate / rollback target
