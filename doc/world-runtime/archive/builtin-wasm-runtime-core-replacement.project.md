> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：删除旧 `agent_world_builtin_wasm` 并替换为新流程 项目管理文档

## 任务拆解
- [x] RCR-1 文档落地（设计文档 + 项目管理文档）。
- [x] RCR-2 代码迁移：删除旧 crate，切换到新 runtime core crate，并完成回归。

## 依赖
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world/src/runtime/*`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：RCR-2 已完成。
- 最近更新：旧 `agent_world_builtin_wasm` 已删除并由 `agent_world_builtin_wasm_runtime` 替代；m1/m4 sync+check 与 required-tier 编译通过（2026-02-17）。
