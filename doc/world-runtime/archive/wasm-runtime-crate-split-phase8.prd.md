# Agent World Runtime：WASM 运行时边界收敛（Phase 8）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime/runtime-integration.md`、`doc/world-runtime/wasm/wasm-interface.md` 与对应源码实现为准。


## 1. Executive Summary
- 去除 `agent_world` 中对沙箱能力的兼容门面（`runtime/sandbox.rs`），避免主 crate 继续承载 WASM ABI/执行器类型转发职责。
- 统一 `agent_world` 内部调用路径：WASM 协议类型直接来自 `agent_world_wasm_abi`，执行器实现直接来自 `agent_world_wasm_executor`。
- 为后续继续拆分 `agent_world` 提供更清晰的 crate 边界（主 crate 只保留 world/runtime 编排职责）。

## 2. User Experience & Functionality
### In Scope
- 删除 `crates/agent_world/src/runtime/sandbox.rs` 与 `runtime::mod` 中对应导出。
- 将 `agent_world` 内部 runtime/simulator/tests 的 `ModuleCall* / ModuleOutput / ModuleSandbox` 引用切换到 `agent_world_wasm_abi`。
- 将测试侧 `FixedSandbox / WasmExecutor / WasmExecutorConfig` 明确从 `agent_world_wasm_executor` 引用。
- 完成 `agent_world` crate 编译与回归测试，验证拆分后行为一致。

### Out of Scope
- 新增沙箱语义、变更执行器能力或修改 Wasmtime 后端行为。
- 调整外部业务协议（治理、快照、分布式网络）或新增分布式路径。
- 在本阶段新增新的 sandbox crate（先做门面移除与边界收敛）。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
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

## 5. Risks & Roadmap
- **R8-1**：移除 `runtime/sandbox.rs` 门面并完成调用方改造。
- **R8-2**：完成 `agent_world` 编译与测试回归。

### Technical Risks
- 依赖旧导出路径（`agent_world::runtime::*` 沙箱类型）的调用方会在编译期失败，需要同步改造。
- 拆除门面后，测试/模拟层导入路径变化较多，存在遗漏风险；需用全量编译和回归测试兜底。

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
