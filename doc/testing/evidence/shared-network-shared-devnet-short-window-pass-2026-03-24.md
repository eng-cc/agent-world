# Shared Network `shared_devnet` Short-Window Pass Evidence (2026-03-24)

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
  - 在同一 `candidate_id=shared-devnet-20260324-05` 上完成真实 S9/S10 short-window rehearsal，把 `short_window_longrun` 从 `partial` 提升到 `pass`。

## 执行范围
- `window_id`:
  - `shared-devnet-20260324-06`
- `candidate_id`:
  - `shared-devnet-20260324-05`
- candidate bundle:
  - `output/release-candidates/shared-devnet-20260324-05.json`
- git commit:
  - `50257fca76de317a605ff51d1dba3b584deb3759`
- bundle dir:
  - `output/release/game-launcher-shared-devnet-20260324-02/`
- rehearsal root:
  - `output/shared-network/shared-devnet-20260324-06/`

## 执行命令
- shared-devnet window:
  - `./scripts/shared-devnet-rehearsal.sh --window-id shared-devnet-20260324-06 --candidate-bundle output/release-candidates/shared-devnet-20260324-05.json --bundle-dir output/release/game-launcher-shared-devnet-20260324-02 --viewer-port 4200 --live-bind 127.0.0.1:5150 --web-bind 127.0.0.1:5140 --release-gate-mode dry-run --web-mode execute --headless-mode execute --pure-api-mode execute --governance-mode evidence --governance-window-evidence-ref doc/testing/evidence/governance-registry-live-world-drill-finality-2026-03-24.md --longrun-mode execute --s9-base-port 5910 --s10-base-port 6110`

## 关键产物
- multi-entry evidence:
  - `output/shared-network/shared-devnet-20260324-06/multi-entry-summary.md`
- longrun evidence:
  - `output/shared-network/shared-devnet-20260324-06/longrun/s9/20260324-174455/summary.md`
  - `output/shared-network/shared-devnet-20260324-06/longrun/s10/20260324-174959/summary.md`
  - `output/shared-network/shared-devnet-20260324-06/longrun-summary.md`
- lane scaffold:
  - `output/shared-network/shared-devnet-20260324-06/lanes.shared_devnet.tsv`
- gate summary:
  - `output/shared-network/shared-devnet-20260324-06/gate/shared_devnet-20260324-175501/summary.md`
  - `output/shared-network/shared-devnet-20260324-06/gate/shared_devnet-20260324-175501/summary.json`
- liveops records:
  - `doc/testing/evidence/shared-network-shared-devnet-short-window-promotion-record-2026-03-24.md`
  - `doc/testing/evidence/shared-network-shared-devnet-short-window-incident-2026-03-24.md`

## QA 结果
- `candidate_bundle_integrity=pass`
- `multi_entry_closure=pass`
- `governance_live_drill=pass`
- `short_window_longrun=pass`
- `shared_access=partial`
- `rollback_target_ready=partial`
- overall:
  - `gate_result=partial`
  - `promotion_recommendation=hold_promotion`

## 结论
- `shared_devnet` 现在已经把所有纯工程内可执行 lane 都推进到 `pass`：
  - `candidate_bundle_integrity`
  - `multi_entry_closure`
  - `governance_live_drill`
  - `short_window_longrun`
- 剩余 blocker 已经只收敛到两条：
  - 真实 `shared_access`
  - formal `rollback_target_ready`
