# Agent World Runtime：WASM 运行时边界收敛（Phase 8）

## 目标
- 去除 `agent_world` 中对沙箱能力的兼容门面（`runtime/sandbox.rs`），避免主 crate 继续承载 WASM ABI/执行器类型转发职责。
- 统一 `agent_world` 内部调用路径：WASM 协议类型直接来自 `agent_world_wasm_abi`，执行器实现直接来自 `agent_world_wasm_executor`。
- 为后续继续拆分 `agent_world` 提供更清晰的 crate 边界（主 crate 只保留 world/runtime 编排职责）。

## 范围
### In Scope
- 删除 `crates/agent_world/src/runtime/sandbox.rs` 与 `runtime::mod` 中对应导出。
- 将 `agent_world` 内部 runtime/simulator/tests 的 `ModuleCall* / ModuleOutput / ModuleSandbox` 引用切换到 `agent_world_wasm_abi`。
- 将测试侧 `FixedSandbox / WasmExecutor / WasmExecutorConfig` 明确从 `agent_world_wasm_executor` 引用。
- 完成 `agent_world` crate 编译与回归测试，验证拆分后行为一致。

### Out of Scope
- 新增沙箱语义、变更执行器能力或修改 Wasmtime 后端行为。
- 调整外部业务协议（治理、快照、分布式网络）或新增分布式路径。
- 在本阶段新增新的 sandbox crate（先做门面移除与边界收敛）。

## 接口 / 数据
- ABI 类型来源统一为 `agent_world_wasm_abi`：
  - `ModuleCallInput`
  - `ModuleCallRequest`
  - `ModuleCallFailure`
  - `ModuleOutput`
  - `ModuleSandbox`
- 执行器类型来源统一为 `agent_world_wasm_executor`：
  - `FixedSandbox`
  - `WasmExecutor`
  - `WasmExecutorConfig`
- `agent_world::runtime` 不再对以上沙箱/调用类型做 re-export。

## 里程碑
- **R8-1**：移除 `runtime/sandbox.rs` 门面并完成调用方改造。
- **R8-2**：完成 `agent_world` 编译与测试回归。

## 风险
- 依赖旧导出路径（`agent_world::runtime::*` 沙箱类型）的调用方会在编译期失败，需要同步改造。
- 拆除门面后，测试/模拟层导入路径变化较多，存在遗漏风险；需用全量编译和回归测试兜底。
