# Agent World Runtime：WASM SDK no_std 优先化 项目管理文档

## 任务拆解
- [x] NSDK-1 文档落地（设计文档 + 项目管理文档）。
- [ ] NSDK-2 代码迁移：`agent_world_wasm_sdk` no_std 化与回归验证。

## 依赖
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world/src/runtime/*`
- `third_party/agent-os/crates/aos-wasm-sdk`（参考）

## 状态
- 当前阶段：NSDK-1 已完成，NSDK-2 进行中。
- 最近更新：新增 no_std 迁移设计与任务拆解（2026-02-17）。
