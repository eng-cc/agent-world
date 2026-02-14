# Agent World Runtime：WASM 运行时激进迁移（项目管理文档，Phase 6）

## 任务拆解
- [x] R6-0 输出设计文档（`doc/world-runtime/wasm-runtime-crate-split-phase6.md`）
- [x] R6-0 输出项目管理文档（本文件）
- [x] R6-1 提取 `ModuleStore` 文件存储实现到独立 crate 并回归

## 依赖
- `crates/agent_world/src/runtime/module_store.rs`
- `crates/agent_world/src/runtime/world/persistence.rs`
- `crates/agent_world_wasm_abi/src/lib.rs`
- `crates/agent_world_wasm_store/src/lib.rs`
- workspace `Cargo.toml`

## 状态
- 当前阶段：R6 完成（`ModuleStore` 文件存储实现已拆到独立 crate）。
- 最近更新：2026-02-14（完成 R6-1 回归）
