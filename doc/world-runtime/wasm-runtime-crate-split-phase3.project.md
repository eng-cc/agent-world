# Agent World Runtime：WASM 运行时激进迁移（项目管理文档）

## 任务拆解
- [x] R3-0 输出设计文档（`doc/world-runtime/wasm-runtime-crate-split-phase3.md`）
- [x] R3-0 输出项目管理文档（本文件）
- [x] R3-1 迁移 `ModuleArtifact/ModuleCache` 到 `agent_world_wasm_abi` 并回归
- [x] R3-2 迁移 `agent_world_net::ModuleArtifact` 到 ABI 统一定义并回归

## 依赖
- `crates/agent_world_wasm_abi/src/lib.rs`
- `crates/agent_world/src/runtime/modules.rs`
- `crates/agent_world_net/src/lib.rs`

## 状态
- 当前阶段：R3 完成（WASM 工件/缓存类型统一到 ABI crate）。
- 最近更新：2026-02-13（完成 net crate 工件类型迁移）
