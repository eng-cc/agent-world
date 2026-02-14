# Agent World Runtime：WASM 运行时拆分后测试加固（项目管理文档）

## 任务拆解
- [x] R2-0 输出设计文档（`doc/world-runtime/wasm-runtime-crate-split-phase2.md`）
- [x] R2-0 输出项目管理文档（本文件）
- [x] R2-1 补齐 `agent_world_wasm_router` 单元测试并回归
- [x] R2-2 补齐 `agent_world_wasm_executor` 单元测试并回归

## 依赖
- `crates/agent_world_wasm_router/src/lib.rs`
- `crates/agent_world_wasm_executor/src/lib.rs`
- `doc/world-runtime/wasm-runtime-crate-split.md`

## 状态
- 当前阶段：R2 完成（router/executor 测试补齐并回归通过）。
- 最近更新：2026-02-13（完成 executor crate 测试补齐）
