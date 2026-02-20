# Agent World Runtime：WASM 运行时边界收敛（Phase 8 项目管理）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 任务拆解
- [x] R8-0：输出 Phase 8 设计文档与项目管理文档。
- [x] R8-1：删除 `runtime/sandbox.rs` 与 `runtime::mod` 对应导出，调用方切换为直接依赖 ABI/Executor crate。
- [x] R8-2：完成 `agent_world` 编译与回归测试（`cargo check` + `cargo test`，`--features wasmtime`）。

## 依赖
- `doc/world-runtime/archive/wasm-runtime-crate-split-phase8.md`
- `crates/agent_world`
- `crates/agent_world_wasm_abi`
- `crates/agent_world_wasm_executor`

## 状态
- 当前阶段：R8 已完成（R8-0 ~ R8-2）。
- 下一步：评估是否继续把沙箱相关高层封装进一步下沉到独立 crate（若立项则进入 R9）。
- 最近更新：2026-02-14
