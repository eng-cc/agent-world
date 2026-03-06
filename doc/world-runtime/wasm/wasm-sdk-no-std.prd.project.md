# Agent World Runtime：WASM SDK no_std 优先化 项目管理文档

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] T-MIG-20260303 (PRD-ENGINEERING-006): 逐篇阅读旧文档并完成人工重写迁移到 `.prd` 命名。
- [x] NSDK-1 文档落地（设计文档 + 项目管理文档）。
- [x] NSDK-2 代码迁移：`agent_world_wasm_sdk` no_std 化与回归验证。

## 依赖
- doc/world-runtime/wasm/wasm-sdk-no-std.prd.md
- `crates/agent_world_wasm_sdk`
- `crates/agent_world_builtin_wasm_modules/*`
- `crates/agent_world/src/runtime/*`
- `third_party/agent-os/`（参考）

## 状态
- 当前阶段：NSDK-2 已完成。
- 最近更新：`agent_world_wasm_sdk` 完成 no_std 优先迁移（默认 no_std，显式声明 `std` feature），SDK 单测与 wasm32/required-tier 编译通过（2026-02-17）。
