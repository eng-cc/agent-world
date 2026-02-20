> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Builtin WASM 生命周期 Trait 与 SDK 项目管理文档

## 任务拆解
- [x] LIFESDK-1 设计与项目文档落地。
- [x] LIFESDK-2 代码迁移：新增 `agent_world_wasm_sdk` + builtin wasm 模块接入生命周期 trait。

## 依赖
- `third_party/agent-os/crates/aos-wasm-sdk`（参考）
- `crates/agent_world_builtin_wasm_runtime`
- `crates/agent_world_builtin_wasm_modules/*`
- `scripts/build-builtin-wasm-modules.sh`

## 状态
- 当前阶段：LIFESDK-2 已完成。
- 最近更新：完成 SDK crate、23 个模块生命周期 trait 接入、构建同步与回归检查（2026-02-17）。
