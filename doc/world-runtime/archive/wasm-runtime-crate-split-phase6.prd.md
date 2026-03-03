# Agent World Runtime：WASM 运行时激进迁移（设计文档，Phase 6）

> [!WARNING]
> 归档状态：**过时设计（仅保留历史记录）**  
> 归档日期：2026-02-17  
> 说明：本文档描述的迁移阶段已完成并并入当前实现，文中的阶段性任务与兼容路径不再作为现行方案。当前设计以 `doc/world-runtime/runtime-integration.md`、`doc/world-runtime/wasm-interface.md` 与对应源码实现为准。


## 1. Executive Summary
- 继续收敛 `agent_world` 主 crate 的 WASM 运行时职责，将模块文件存储实现从 runtime 中抽离到独立 crate。
- 为后续 runtime 最小化与跨 crate 复用做准备，让 `agent_world` 保留编排与兼容层职责。

## 2. User Experience & Functionality
### In Scope（本次）
- 新增独立 crate（暂定 `agent_world_wasm_store`），承载模块工件/元数据/注册表的文件持久化实现。
- 将现有 `ModuleStore` 核心文件读写逻辑迁移到新 crate。
- `agent_world` 侧保留 `ModuleStore` 对外 API（可通过轻量包装 + 错误映射维持兼容）。
- 回归验证 `module_store` 与 world 持久化相关测试路径。

### Out of Scope（本次不做）
- 改动模块存储目录结构（`module_registry.json` + `modules/*.wasm` + `*.meta.json` 保持不变）。
- 修改治理流程、模块注册语义或事件 schema。
- 引入远端对象存储/分布式存储后端。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 新 crate 依赖 `agent_world_wasm_abi` 复用 `ModuleManifest/ModuleRegistry/ModuleRecord`。
- `agent_world` 根导出继续暴露 `ModuleStore`，尽量不影响上层调用方。
- 错误语义保持一致（版本不匹配、I/O/序列化失败）。

## 5. Risks & Roadmap
- **R6-0**：文档与任务拆解完成。
- **R6-1**：`ModuleStore` 文件存储实现拆到独立 crate 并回归通过。

### Technical Risks
- 错误映射若遗漏，可能导致现有测试断言和上层错误处理行为变化。
- 抽离后若路径/原子写细节偏移，可能影响模块工件持久化稳定性。

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
