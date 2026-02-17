# Agent World Runtime：WASM 运行时拆分后测试加固（项目管理文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


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
