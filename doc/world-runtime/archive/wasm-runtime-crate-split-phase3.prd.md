# Agent World Runtime：WASM 运行时激进迁移（设计文档）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 1. Executive Summary
- 在 WRS（拆 crate）与 R2（测试加固）之后，继续将仍驻留在 `agent_world` 的 WASM 通用类型下沉到独立 crate。
- 先迁移高复用、低耦合的工件与缓存模型，减少 runtime 主 crate 的“类型负担”。

## 2. User Experience & Functionality
### In Scope（本次）
- 将 `ModuleArtifact` 与 `ModuleCache` 从 `agent_world` 迁移到 `agent_world_wasm_abi`。
- `agent_world` 侧改为直接复用 ABI crate 导出类型（保持外部接口兼容）。
- `agent_world_net` 侧复用同一个 `ModuleArtifact` 定义，去除重复结构体。
- 为迁移后的缓存类型补齐 crate 内单元测试。

### Out of Scope（本次不做）
- 调整 `ModuleManifest/ModuleRegistry/ModuleEvent` 的归属。
- 变更分布式协议结构或 DHT 存储键设计。
- 调整执行器行为、路由语义或治理流程。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- `agent_world_wasm_abi` 新增并对外导出：
  - `ModuleArtifact`
  - `ModuleCache`
- 兼容策略：
  - `agent_world` 继续通过 runtime 导出同名类型，不引入上层 API 破坏。
  - `agent_world_net` 对外 `ModuleArtifact` 字段保持不变（`wasm_hash/bytes`）。

## 5. Risks & Roadmap
- **R3-0**：文档与任务拆解完成。
- **R3-1**：`ModuleArtifact/ModuleCache` 迁移到 ABI crate 并完成回归。
- **R3-2**：`agent_world_net` 复用 ABI `ModuleArtifact` 并完成回归。

### Technical Risks
- 类型迁移涉及跨 crate 导出路径，若重导出处理不当可能影响编译边界。
- 缓存行为若在迁移中发生细微变化，会影响模块加载路径稳定性。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
