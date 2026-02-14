# Agent World Runtime：WASM 运行时激进迁移（设计文档）

## 目标
- 在 WRS（拆 crate）与 R2（测试加固）之后，继续将仍驻留在 `agent_world` 的 WASM 通用类型下沉到独立 crate。
- 先迁移高复用、低耦合的工件与缓存模型，减少 runtime 主 crate 的“类型负担”。

## 范围

### In Scope（本次）
- 将 `ModuleArtifact` 与 `ModuleCache` 从 `agent_world` 迁移到 `agent_world_wasm_abi`。
- `agent_world` 侧改为直接复用 ABI crate 导出类型（保持外部接口兼容）。
- `agent_world_net` 侧复用同一个 `ModuleArtifact` 定义，去除重复结构体。
- 为迁移后的缓存类型补齐 crate 内单元测试。

### Out of Scope（本次不做）
- 调整 `ModuleManifest/ModuleRegistry/ModuleEvent` 的归属。
- 变更分布式协议结构或 DHT 存储键设计。
- 调整执行器行为、路由语义或治理流程。

## 接口 / 数据
- `agent_world_wasm_abi` 新增并对外导出：
  - `ModuleArtifact`
  - `ModuleCache`
- 兼容策略：
  - `agent_world` 继续通过 runtime 导出同名类型，不引入上层 API 破坏。
  - `agent_world_net` 对外 `ModuleArtifact` 字段保持不变（`wasm_hash/bytes`）。

## 里程碑
- **R3-0**：文档与任务拆解完成。
- **R3-1**：`ModuleArtifact/ModuleCache` 迁移到 ABI crate 并完成回归。
- **R3-2**：`agent_world_net` 复用 ABI `ModuleArtifact` 并完成回归。

## 风险
- 类型迁移涉及跨 crate 导出路径，若重导出处理不当可能影响编译边界。
- 缓存行为若在迁移中发生细微变化，会影响模块加载路径稳定性。
