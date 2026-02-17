# Agent World Runtime：Builtin WASM 模块独立工程化（MIP）项目管理文档

## 任务拆解
- [x] MIP-1 设计与项目管理文档落地（本文件 + 设计文档）。
- [ ] MIP-2 代码迁移：模块独立 crate + 构建映射链路改造。
- [ ] MIP-3 收口：hash/DistFS 同步校验、回归测试、文档/devlog 更新。

## 依赖
- `crates/agent_world_builtin_wasm`
- `scripts/build-builtin-wasm-modules.sh`
- `scripts/build-wasm-module.sh`
- `tools/wasm_build_suite`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：MIP-1 已完成，MIP-2 进行中。
- 最近更新：新增独立工程化设计与任务拆解（2026-02-17）。
