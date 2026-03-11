# scripts PRD

审计轮次: 6

## 目标
- 建立 scripts 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 scripts 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 scripts 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/scripts/project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/scripts/prd.md`
- 项目管理入口: `doc/scripts/project.md`
- 文件级索引: doc/scripts/prd.index.md
- 追踪主键: `PRD-SCRIPTS-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 自动化脚本覆盖构建、测试、发布与调试，但职责边界和使用规范分散，导致脚本重叠、入口混乱和维护成本上升。
- Proposed Solution: scripts PRD 统一定义脚本分层（开发、CI、发布、排障）、调用约束、兼容策略与验证标准。
- Success Criteria:
  - SC-1: 核心脚本均有明确 owner、输入输出约定与失败语义。
  - SC-2: 新增脚本在合并前通过语法/参数最小校验。
  - SC-3: 脚本入口重复率下降并保留稳定主入口。
  - SC-4: 脚本任务 100% 映射到 PRD-SCRIPTS-ID。

## 2. User Experience & Functionality
- User Personas:
  - 开发者：需要可预期的脚本入口与错误提示。
  - CI 维护者：需要稳定脚本接口，减少流水线波动。
  - 排障人员：需要区分常规链路与 fallback 工具链路。
- User Scenarios & Frequency:
  - 日常开发执行：开发者每次本地验证时使用主入口脚本。
  - CI 流水线运行：每次合并与 nightly 执行。
  - 故障排查：出现异常时按 fallback 规则执行诊断脚本。
  - 脚本契约更新：每周巡检并同步参数文档。
- User Stories:
  - PRD-SCRIPTS-001: As a 开发者, I want stable script entry points, so that daily workflows are reliable.
  - PRD-SCRIPTS-002: As a CI 维护者, I want deterministic script contracts, so that pipeline changes are controlled.
  - PRD-SCRIPTS-003: As a 排障人员, I want explicit fallback tooling rules, so that issue triage is faster.
- Critical User Flows:
  1. Flow-SCR-001: `调用主入口脚本 -> 执行检查/测试 -> 输出结构化结果`
  2. Flow-SCR-002: `CI 触发脚本 -> 失败定位到参数/环境 -> 修复后重跑`
  3. Flow-SCR-003: `常规链路无法复现 -> 触发 fallback 工具 -> 采集诊断证据`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 脚本主入口 | 脚本名、参数、返回码、输出路径 | 执行并输出标准化结果 | `idle -> running -> success/failed` | 按命令类型分层执行 | 所有人可执行 |
| 参数契约 | 必填参数、默认值、失败语义 | 参数校验失败即阻断 | `validating -> accepted/rejected` | 必填项优先校验 | 维护者可更新契约 |
| fallback 规则 | 触发条件、替代脚本、产物要求 | 满足条件后才允许 fallback | `normal -> fallback -> diagnosed` | 常规链路优先 | 仅排障场景允许触发 |
- Acceptance Criteria:
  - AC-1: scripts PRD 明确脚本分类、入口、约束。
  - AC-2: scripts project 文档维护脚本治理任务。
  - AC-3: 与 `doc/scripts/precommit/pre-commit.prd.md`、`testing-manual.md` 口径一致。
  - AC-4: `capture-viewer-frame.sh` 被明确为 fallback 链路使用。
- Non-Goals:
  - 不在 scripts PRD 中替代业务功能设计。
  - 不承诺所有历史脚本长期向后兼容。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: Bash 校验、脚本帮助文档、CI 调用链路。
- Evaluation Strategy: 以脚本失败定位时长、重复脚本数量、CI 脚本稳定性趋势评估。

## 4. Technical Specifications
- Architecture Overview: scripts 模块是工程自动化执行层，向开发、测试、发布提供可组合命令入口，强调“单一职责 + 明确输出”。
- Integration Points:
  - `scripts/`
  - `doc/scripts/precommit/`
  - `doc/scripts/viewer-tools/`
  - `doc/scripts/wasm/`
  - `testing-manual.md`
  - `.github/workflows/*`
- Edge Cases & Error Handling:
  - 参数缺失：立即失败并打印最小可执行示例。
  - 依赖缺失：输出依赖安装提示与环境检查命令。
  - 超时：长脚本超时后输出中间进度并建议重试策略。
  - 权限不足：不可写目录或权限异常时给出路径修复建议。
  - 并发冲突：同产物目录并发执行时强制隔离输出。
  - fallback 误用：未满足触发条件时拒绝 fallback。
- Non-Functional Requirements:
  - NFR-SCR-1: 核心脚本具备可读帮助信息与失败语义说明。
  - NFR-SCR-2: 主入口脚本在 Linux/macOS 环境可执行一致。
  - NFR-SCR-3: CI 脚本接口稳定，破坏性改动需预告与回归。
  - NFR-SCR-4: 脚本默认输出不得包含敏感信息。
  - NFR-SCR-5: fallback 流程必须可追溯到故障诊断记录。
- Security & Privacy: 脚本不得在默认输出中泄漏密钥；涉及网络调用时需要显式参数与最小权限。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化脚本分层与主入口规范。
  - v1.1: 增加高频脚本的契约测试与参数回归。
  - v2.0: 建立脚本治理仪表（稳定性、复用率、故障恢复时间）。
- Technical Risks:
  - 风险-1: 历史脚本行为差异导致切换成本。
  - 风险-2: 入口过多导致文档与实际调用脱节。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-SCRIPTS-001 | TASK-SCRIPTS-001/002/005 | `test_tier_required` | 脚本分层与入口清单核验 | 日常开发链路稳定性 |
| PRD-SCRIPTS-002 | TASK-SCRIPTS-002/003/005 | `test_tier_required` + `test_tier_full` | 参数契约与失败语义回归 | CI 稳定性与故障定位效率 |
| PRD-SCRIPTS-003 | TASK-SCRIPTS-003/004/005 | `test_tier_required` | fallback 使用条件抽样检查 | 排障闭环和风险控制 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-SCR-001 | 主入口 + fallback 分层治理 | 全脚本平级使用 | 分层更利于稳定维护。 |
| DEC-SCR-002 | 参数契约显式化 | 依赖隐式约定 | 可减少 CI 误用与回归。 |
| DEC-SCR-003 | fallback 仅在受控场景启用 | 默认对所有场景开放 | 可避免过度依赖应急链路。 |
