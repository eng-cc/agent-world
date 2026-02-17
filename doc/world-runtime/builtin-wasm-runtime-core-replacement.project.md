# Agent World Runtime：删除旧 `agent_world_builtin_wasm` 并替换为新流程 项目管理文档

## 任务拆解
- [x] RCR-1 文档落地（设计文档 + 项目管理文档）。
- [ ] RCR-2 代码迁移：删除旧 crate，切换到新 runtime core crate，并完成回归。

## 依赖
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world/src/runtime/*`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：RCR-1 已完成，RCR-2 待执行。
- 最近更新：确认迁移策略为“旧 crate 下线 + 新 runtime core crate 承接逻辑”（2026-02-17）。
