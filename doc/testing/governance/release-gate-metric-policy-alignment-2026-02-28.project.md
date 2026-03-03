# 发布门禁指标策略对齐（2026-02-28）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 正向修复：`world_chain_runtime` 接入 reward runtime worker，并在 S9/S10 消费真实 reward 指标。
- [x] T2 回归验证：语法检查 + `cargo check/test` + 复跑 S9/S10 发布门禁命令。
- [x] T3 文档收口：更新 `testing-manual.md`、补 devlog、项目结项。

## 依赖
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_chain_runtime/reward_runtime_worker.rs`
- `scripts/p2p-longrun-soak.sh`
- `scripts/s10-five-node-game-soak.sh`
- `testing-manual.md`
- `.tmp/release_gate_s10/20260228-222029/summary.json`
- `.tmp/release_gate_p2p/20260228-225152/summary.json`

## 状态
- 当前阶段：已完成（T0~T3）。
- 当前任务：无。
