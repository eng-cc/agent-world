# Agent World Runtime：WASM 模块设计对齐增强（项目管理文档）

## 任务拆解
- [x] AOSA-1 ABI/Schema 合约约束增强（manifest 字段与 shadow 校验）。
- [x] AOSA-2 cap slot 绑定（manifest `cap_slots` + output `cap_slot` 解析）。
- [x] AOSA-3 pure 模块策略插件链路（effect 前置判定）。
- [ ] AOSA-5 `ModuleContext` 元信息增强与贯通。
- [ ] AOSA-6 `WasmExecutor` 磁盘编译缓存（wasmtime serialized module）。

## 依赖
- 现有 wasm ABI：`crates/agent_world_wasm_abi/src/lib.rs`
- 现有 runtime 模块执行链路：`crates/agent_world/src/runtime/world/module_runtime.rs`
- 执行器实现：`crates/agent_world_wasm_executor/src/lib.rs`
- 参考实现（只读）：`third_party/agent-os`

## 状态
- 当前阶段：AOSA-5 进行中。
- 最近更新：AOSA-3 已完成（pure policy hook 链路 + allow/deny 测试）。
