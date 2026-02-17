# Agent World Runtime：WASM 运行时拆分后测试加固（设计文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 目标
- 在完成 WRS 拆 crate 后，补齐 `agent_world_wasm_router` 与 `agent_world_wasm_executor` 的单元测试覆盖。
- 将订阅匹配/过滤器与执行器基础行为转为 crate 内闭环验证，降低回归风险。

## 范围

### In Scope（本次）
- 为 `agent_world_wasm_router` 增加 stage 校验、pattern 匹配、filters 匹配/校验的单元测试。
- 为 `agent_world_wasm_executor` 增加 `FixedSandbox` 与缓存边界行为的单元测试。
- 保持外部 API 与运行时行为不变。

### Out of Scope（本次不做）
- 调整 ABI 结构定义或模块协议版本。
- 改造 `World` 路由流程与治理流程。
- 变更 wasmtime 执行策略与资源限制语义。

## 接口 / 数据
- 无新增对外接口。
- 测试覆盖目标：
  - `agent_world_wasm_router::module_subscribes_to_event/action`
  - `agent_world_wasm_router::validate_subscription_stage/filters`
  - `agent_world_wasm_executor::FixedSandbox`
  - `agent_world_wasm_executor` 编译缓存边界（`max_cache_entries=0`）

## 里程碑
- **R2-0**：文档与任务拆解完成。
- **R2-1**：router crate 测试补齐并通过。
- **R2-2**：executor crate 测试补齐并通过。

## 风险
- 过滤器测试样例不足可能导致行为漂移未被捕获。
- executor 的 wasmtime 分支若仅在 feature 下测试，需确保 CI 覆盖该路径。
