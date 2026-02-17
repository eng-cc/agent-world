# Agent World Runtime：WASM SDK Wire 类型收敛项目管理文档

## 任务拆解
- [x] WIRESDK-1 设计文档与项目管理文档落地。
- [ ] WIRESDK-2 代码迁移：SDK 增加通用 wire 类型与 helper，23 模块改为复用 SDK。

## 依赖
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_builtin_wasm_modules/*`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：WIRESDK-1 已完成，WIRESDK-2 进行中。
- 最近更新：完成 wire 收敛方案设计与任务拆解（2026-02-17）。
