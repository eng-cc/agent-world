# Agent World Runtime：Builtin WASM 生命周期 Trait 与 SDK 项目管理文档

## 任务拆解
- [x] LIFESDK-1 设计与项目文档落地。
- [ ] LIFESDK-2 代码迁移：新增 `agent_world_wasm_sdk` + builtin wasm 模块接入生命周期 trait。

## 依赖
- `third_party/agent-os/crates/aos-wasm-sdk`（参考）
- `crates/agent_world_builtin_wasm`
- `crates/agent_world_builtin_wasm_modules/*`
- `scripts/build-builtin-wasm-modules.sh`

## 状态
- 当前阶段：LIFESDK-1 已完成，LIFESDK-2 待执行。
- 最近更新：新增 lifecycle+sdk 设计方案并进入代码迁移准备（2026-02-17）。
