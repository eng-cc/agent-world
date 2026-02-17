# Agent World Runtime：Builtin WASM 业务逻辑下沉到模块工程 项目管理文档

## 任务拆解
- [x] MBM-1 文档落地（设计文档 + 项目管理文档）。
- [ ] MBM-2 代码迁移：23 模块业务逻辑下沉、runtime 去业务分发、构建与回归校验。

## 依赖
- `crates/agent_world_builtin_wasm_runtime`
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world_wasm_sdk`
- `scripts/build-builtin-wasm-modules.sh`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：MBM-1 已完成，MBM-2 进行中。
- 最近更新：已完成业务逻辑下沉迁移方案设计与任务拆解（2026-02-17）。
