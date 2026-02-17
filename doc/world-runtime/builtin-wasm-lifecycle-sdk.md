# Agent World Runtime：Builtin WASM 生命周期 Trait 与 SDK 设计

## 目标
- 参考 `third_party/agent-os` 的 wasm SDK 思路，在 Agent World 中补齐统一的 wasm 模块生命周期抽象。
- 用 trait 约束 wasm 模块生命周期，统一 `alloc/reduce/call` 导出与调用编排。
- 新增独立 crate `agent_world_wasm_sdk`，为 wasm 模块提供基础辅助能力（生命周期 dispatch、导出宏、内存分配辅助）。

## 范围

### In Scope
- 新增 `crates/agent_world_wasm_sdk`。
- 定义生命周期 trait（含 `on_init`、`on_teardown`、`on_reduce`、`on_call`）。
- 在 SDK 中提供：
  - 标准分配辅助函数；
  - `reduce/call` 生命周期 dispatch；
  - 导出 `alloc/reduce/call` 的宏。
- 将 `crates/agent_world_builtin_wasm_modules/*` 从“手写三导出函数”迁移为“实现生命周期 trait + SDK 宏导出”。

### Out of Scope
- 不改 runtime ABI 协议（仍然是 `alloc/reduce/call`）。
- 不改 builtin 模块业务逻辑（规则/经济/身体/记忆等逻辑保持不变）。
- 不改 `third_party/agent-os` 代码（只读参考）。

## 接口 / 数据
- 新增 crate：`crates/agent_world_wasm_sdk`。
- 生命周期 trait（草案）：
  - `trait WasmModuleLifecycle: Default`
  - `fn module_id(&self) -> &'static str`
  - `fn alloc(&mut self, len: i32) -> i32`（可重写）
  - `fn on_init(&mut self, stage: LifecycleStage)`
  - `fn on_teardown(&mut self, stage: LifecycleStage)`
  - `fn on_reduce(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32)`
  - `fn on_call(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32)`（默认复用 `on_reduce`）
- 生命周期阶段枚举：`LifecycleStage::{Reduce, Call}`。
- SDK dispatch：
  - `dispatch_reduce::<M>(ptr, len)`
  - `dispatch_call::<M>(ptr, len)`
- SDK 导出宏：
  - `export_wasm_module!(Type)` 统一导出 `alloc/reduce/call`。

## 里程碑
- LIFESDK-1：设计文档与项目文档落地。
- LIFESDK-2：代码迁移（SDK crate + builtin wasm 模块接入）与回归验证。

## 风险
- trait 落地后如果模块未接入宏，可能出现导出不一致：通过批量迁移与编译检查收敛。
- 生命周期钩子增加后若未来引入状态副作用，可能影响确定性：当前仅定义空实现，保持纯转发。
- SDK 抽象过重导致模块样板增加：通过导出宏压缩模板代码。
