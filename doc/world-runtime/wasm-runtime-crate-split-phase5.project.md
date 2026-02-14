# Agent World Runtime：WASM 运行时激进迁移（项目管理文档，Phase 5）

## 任务拆解
- [x] R5-0 输出设计文档（`doc/world-runtime/wasm-runtime-crate-split-phase5.md`）
- [x] R5-0 输出项目管理文档（本文件）
- [x] R5-1 迁移模块注册表与生命周期事件类型到 `agent_world_wasm_abi` 并回归

## 依赖
- `crates/agent_world_wasm_abi/src/lib.rs`
- `crates/agent_world/src/runtime/modules.rs`
- `crates/agent_world/src/runtime/world/module_runtime.rs`
- `crates/agent_world/src/runtime/module_store.rs`

## 状态
- 当前阶段：R5 完成（模块注册表与生命周期事件类型已下沉到 ABI crate）。
- 最近更新：2026-02-14（完成 R5-1 回归）
