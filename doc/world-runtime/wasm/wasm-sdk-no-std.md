# Agent World Runtime：WASM SDK no_std 优先化设计

## 目标
- 将 `crates/agent_world_wasm_sdk` 调整为 no_std 优先，实现与 `third_party/agent-os` wasm sdk 在运行时约束上的基础对齐。
- 保持现有 wasm ABI（`alloc/reduce/call`）和生命周期 trait 接口不变，确保已有模块迁移成本为零。
- 为后续 typed context/effect 能力扩展提供 no_std 基础。

## 范围

### In Scope
- `agent_world_wasm_sdk` crate 启用 `#![cfg_attr(not(any(test, feature = "std")), no_std)]`。
- 将 SDK 内部的 `std` 依赖替换为 `core` / `alloc`。
- 补齐最小回归：SDK 单测、SDK wasm32 编译、`agent_world` required-tier 编译。

### Out of Scope
- 不修改 runtime core 业务逻辑（`agent_world_builtin_wasm_runtime`）。
- 不修改 `alloc/reduce/call` ABI。
- 不引入 typed reducer/context/effect 新接口。

## 接口 / 数据
- crate：`crates/agent_world_wasm_sdk`
- 兼容接口：
  - `LifecycleStage`
  - `WasmModuleLifecycle`
  - `dispatch_reduce` / `dispatch_call`
  - `export_wasm_module!`
- 构建约束：默认 no_std；`test` 或显式 `std` feature 下可使用 std 环境。

## 里程碑
- NSDK-1：设计文档 + 项目管理文档落地。
- NSDK-2：SDK no_std 迁移、回归测试、文档/devlog 收口。

## 风险
- 风险：`cfg(feature = "std")` 未定义导致 check-cfg warning。
  - 缓解：在 `Cargo.toml` 显式声明 `std` feature。
- 风险：no_std 场景下内存分配行为变化。
  - 缓解：保留现有 `default_alloc` 语义并用单测回归。
- 风险：下游模块隐含 std 假设。
  - 缓解：执行 `agent_world` required-tier 编译校验接线不回归。
