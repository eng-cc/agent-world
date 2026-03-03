# 基于 world_chain_runtime 的长跑脚本可用化（2026-02-28）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 改造 `s10-five-node-game-soak.sh` 为可运行的 `world_chain_runtime` 五节点脚本。
- [x] T2 改造 `p2p-longrun-soak.sh` 为可运行的 `world_chain_runtime` 脚本（含 chaos 注入）。
- [x] T3 回归与收口：更新手册口径、补任务日志、结项。

## 依赖
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- 现有长跑脚本产物消费方（summary/timeline）
- `testing-manual.md` 与 `doc/testing/*` 启动口径

## 状态
- 当前阶段：已完成（T0~T3）。
- 当前任务：无（项目已结项）。
