# Agent World Runtime：WASM 运行时激进迁移（项目管理文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 任务拆解
- [x] R3-0 输出设计文档（`doc/world-runtime/archive/wasm-runtime-crate-split-phase3.md`）
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
