# Agent World Runtime：WASM 运行时激进迁移（项目管理文档，Phase 5）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 任务拆解
- [x] R5-0 输出设计文档（`doc/world-runtime/archive/wasm-runtime-crate-split-phase5.md`）
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
