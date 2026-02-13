# Agent World Runtime：WASM 运行时激进拆分（项目管理文档）

## 任务拆解
- [x] WRS-1 输出设计文档（`doc/world-runtime/wasm-runtime-crate-split.md`）
- [x] WRS-1 输出项目管理文档（本文件）
- [ ] WRS-2 新建 `agent_world_wasm_abi` 并迁移 wasm ABI 协议类型/trait
- [ ] WRS-2 新建 `agent_world_wasm_executor` 并迁移执行器实现
- [ ] WRS-2 新建 `agent_world_wasm_router` 并迁移订阅匹配/过滤器逻辑
- [ ] WRS-2 `agent_world` 接入三 crate 并保持 re-export 兼容
- [ ] WRS-3 拆分 `agent_world_builtin_wasm/src/lib.rs` 为目录化模块
- [ ] WRS-3 运行回归测试（check + wasm 路由关键测试）
- [ ] WRS-4 更新总设计文档/总项目文档与 devlog 收口

## 依赖
- `doc/world-runtime/wasm-executor.md`
- `doc/world-runtime/runtime-integration.md`
- `crates/agent_world/src/runtime/sandbox.rs`
- `crates/agent_world/src/runtime/world/module_runtime.rs`
- `crates/agent_world_builtin_wasm/src/lib.rs`

## 状态
- 当前阶段：WRS-1 完成，WRS-2 进行中。
- 最近更新：2026-02-13
