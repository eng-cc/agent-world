# Agent World Runtime：WASM 运行时激进拆分（设计文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 目标
- 将 WASM 运行时能力从 `agent_world` 主 crate 中拆出，形成稳定、可复用、可测试的独立 crate 边界。
- 降低 `runtime` 与 `wasmtime`、ABI 协议、订阅路由解析之间的耦合，缩短增量编译路径。
- 在保持外部 API 兼容的前提下，清理超长文件与多职责聚合文件，提升维护效率。

## 范围

### In Scope（本次）
- 新增 `agent_world_wasm_abi`：承载 `ModuleLimits`、`ModuleCall*`、`ModuleOutput`、`ModuleSandbox` 等基础协议类型。
- 新增 `agent_world_wasm_executor`：承载 `FixedSandbox`、`WasmExecutor`、wasmtime 缓存与资源限制实现。
- 新增 `agent_world_wasm_router`：承载事件/动作订阅匹配、过滤器解析与校验逻辑。
- `agent_world` 运行时改为依赖上述 crate，并通过 re-export 保持外部 API 路径兼容。
- 拆分 `agent_world_builtin_wasm` 的超长 `lib.rs`，改为目录化结构与职责分层。

### Out of Scope（本次不做）
- `World` 类型与治理/快照/分布式主流程拆出 `agent_world`。
- 重新设计模块治理协议或更改模块 ABI 版本。
- 更换 wasmtime 引擎或引入多引擎后端。

## 接口 / 数据

### crate 边界
- `agent_world_wasm_abi`
  - `ModuleKind`、`ModuleLimits`
  - `ModuleCallRequest`、`ModuleCallInput`、`ModuleContext`
  - `ModuleOutput`、`ModuleEffectIntent`、`ModuleEmit`
  - `ModuleCallErrorCode`、`ModuleCallFailure`
  - `ModuleSandbox` trait
- `agent_world_wasm_executor`
  - `FixedSandbox`
  - `WasmExecutorConfig`、`WasmEngineKind`、`WasmExecutor`
- `agent_world_wasm_router`
  - `event_kind_label`、`action_kind_label`
  - `module_subscribes_to_event`、`module_subscribes_to_action`
  - `validate_subscription_stage`、`validate_subscription_filters`

### 兼容策略
- `agent_world::runtime` 与 `agent_world` 根导出继续暴露原有符号名。
- `agent_world` 内部将 `ModuleLimits` 等类型源切到 `agent_world_wasm_abi`，尽量不改变调用点语义。

## 里程碑
- **WRS-1**：文档与任务拆解完成（本设计 + 项目管理文档）。
- **WRS-2**：ABI/Executor/Router 三 crate 落地并接入 `agent_world`。
- **WRS-3**：`agent_world_builtin_wasm` 目录化拆分与回归通过。
- **WRS-4**：兼容导出与文档收口完成。

## 风险
- 拆分初期可能出现类型重名或循环依赖，需要严格约束 crate 单向依赖。
- re-export 漏项会造成外部 API 编译失败。
- 订阅过滤器迁移后若行为漂移，可能影响模块路由结果。
