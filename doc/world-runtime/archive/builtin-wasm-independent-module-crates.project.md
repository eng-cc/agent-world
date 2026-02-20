> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Builtin WASM 模块独立工程化（MIP）项目管理文档

## 任务拆解
- [x] MIP-1 设计与项目管理文档落地（本文件 + 设计文档）。
- [x] MIP-2 代码迁移：模块独立 crate + 构建映射链路改造。
- [x] MIP-3 收口：hash/DistFS 同步校验、回归测试、文档/devlog 更新，并下线旧 builtin wasm CI gate。

## 依赖
- `crates/agent_world_builtin_wasm`
- `scripts/build-builtin-wasm-modules.sh`
- `scripts/build-wasm-module.sh`
- `tools/wasm_build_suite`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：MIP-3 已完成。
- 最近更新：完成 m1/m4 hash 与 DistFS 校验、required-tier 编译回归，并在 CI/提交流程移除旧 builtin wasm 同步校验（2026-02-17）。
