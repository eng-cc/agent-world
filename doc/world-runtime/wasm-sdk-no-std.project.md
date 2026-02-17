# Agent World Runtime：WASM SDK no_std 优先化 项目管理文档

## 任务拆解
- [x] NSDK-1 文档落地（设计文档 + 项目管理文档）。
- [x] NSDK-2 代码迁移：`agent_world_wasm_sdk` no_std 化与回归验证。

## 依赖
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world/src/runtime/*`
- `third_party/agent-os/crates/aos-wasm-sdk`（参考）

## 状态
- 当前阶段：NSDK-2 已完成。
- 最近更新：`agent_world_wasm_sdk` 完成 no_std 优先迁移（默认 no_std，显式声明 `std` feature），SDK 单测与 wasm32/required-tier 编译通过（2026-02-17）。
