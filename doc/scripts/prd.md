# scripts PRD

## 目标
- 建立 scripts 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 scripts 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 scripts 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/scripts/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/scripts/prd.md`
- 项目管理入口: `doc/scripts/prd.project.md`
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
- User Stories:
  - PRD-SCRIPTS-001: As a 开发者, I want stable script entry points, so that daily workflows are reliable.
  - PRD-SCRIPTS-002: As a CI 维护者, I want deterministic script contracts, so that pipeline changes are controlled.
  - PRD-SCRIPTS-003: As a 排障人员, I want explicit fallback tooling rules, so that issue triage is faster.
- Acceptance Criteria:
  - AC-1: scripts PRD 明确脚本分类、入口、约束。
  - AC-2: scripts project 文档维护脚本治理任务。
  - AC-3: 与 `doc/scripts/pre-commit.md`、`testing-manual.md` 口径一致。
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
  - `doc/scripts/*.md`
  - `testing-manual.md`
  - `.github/workflows/*`
- Security & Privacy: 脚本不得在默认输出中泄漏密钥；涉及网络调用时需要显式参数与最小权限。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化脚本分层与主入口规范。
  - v1.1: 增加高频脚本的契约测试与参数回归。
  - v2.0: 建立脚本治理仪表（稳定性、复用率、故障恢复时间）。
- Technical Risks:
  - 风险-1: 历史脚本行为差异导致切换成本。
  - 风险-2: 入口过多导致文档与实际调用脱节。
