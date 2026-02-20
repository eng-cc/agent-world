> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Builtin WASM 业务逻辑下沉到模块工程 项目管理文档

## 任务拆解
- [x] MBM-1 文档落地（设计文档 + 项目管理文档）。
- [x] MBM-2 代码迁移：23 模块业务逻辑下沉、runtime 去业务分发、构建与回归校验。

## 依赖
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_wasm_store`
- `scripts/build-builtin-wasm-modules.sh`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：MBM-1 / MBM-2 已全部完成。
- 最近更新：23 模块已完成业务逻辑下沉并仅依赖 SDK，`agent_world_builtin_wasm_runtime` 已删除，runtime 常量改由 `agent_world_wasm_store` 提供；m1/m4 sync+check 与 required-tier 编译回归通过（2026-02-17）。
