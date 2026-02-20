# Agent World Runtime：WASM 运行时激进拆分（项目管理文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 任务拆解
- [x] WRS-1 输出设计文档（`doc/world-runtime/archive/wasm-runtime-crate-split.md`）
- [x] WRS-1 输出项目管理文档（本文件）
- [x] WRS-2 新建 `agent_world_wasm_abi` 并迁移 wasm ABI 协议类型/trait
- [x] WRS-2 新建 `agent_world_wasm_executor` 并迁移执行器实现
- [x] WRS-2 新建 `agent_world_wasm_router` 并迁移订阅匹配/过滤器逻辑
- [x] WRS-2 `agent_world` 接入三 crate 并保持 re-export 兼容
- [x] WRS-3 拆分 `agent_world_builtin_wasm/src/lib.rs` 为目录化模块
- [x] WRS-3 运行回归测试（check + wasm 路由关键测试）
- [x] WRS-4 更新总设计文档/总项目文档与 devlog 收口

## 依赖
- `doc/world-runtime/wasm-executor.md`
- `doc/world-runtime/runtime-integration.md`
- `crates/agent_world/src/runtime/sandbox.rs`
- `crates/agent_world/src/runtime/world/module_runtime.rs`
- `crates/agent_world_builtin_wasm/src/lib.rs`

## 状态
- 当前阶段：WRS-1~WRS-4 全部完成。
- 最近更新：2026-02-13
