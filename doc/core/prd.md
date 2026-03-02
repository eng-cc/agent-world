# core PRD

## 目标
- 建立 core 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 core 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 core 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/core/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/core/prd.md`
- 项目管理入口: `doc/core/prd.project.md`
- 追踪主键: `PRD-CORE-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 当前跨模块规则（文档入口、测试分层、发布口径、追踪主键）在多个文件分散维护，导致跨模块协作容易出现边界不一致。
- Proposed Solution: 以 `doc/core/prd.md` 定义仓库级架构与治理基线，约束各模块 PRD 的公共结构、追踪方式与质量门禁。
- Success Criteria:
  - SC-1: 100% 模块拥有 `prd.md` 与 `prd.project.md` 且可在 `doc/README.md` 导航。
  - SC-2: 新增任务在合并前具备 PRD-ID 映射，抽检覆盖率达到 100%。
  - SC-3: 文档治理检查脚本连续 14 天无结构性失败。
  - SC-4: 模块 PRD 章节结构与 AGENTS.md 约束保持一致。

## 2. User Experience & Functionality
- User Personas:
  - 架构负责人：需要统一定义跨模块约束，降低接口漂移。
  - 发布负责人：需要按统一门禁判断是否可发布。
  - 模块维护者：需要快速定位跨模块依赖与边界。
- User Stories:
  - PRD-CORE-001: As an 架构负责人, I want one canonical governance baseline, so that module design docs stay aligned.
  - PRD-CORE-002: As a 发布负责人, I want enforceable release gates, so that cross-module regressions are prevented.
  - PRD-CORE-003: As a 模块维护者, I want shared PRD conventions, so that review and traceability stay consistent.
- Acceptance Criteria:
  - AC-1: core PRD 明确跨模块约束清单（文档、测试、追踪、发布）。
  - AC-2: core project 文档任务可映射到 PRD-CORE-001/002/003。
  - AC-3: `doc/README.md` 的模块入口与 core PRD 基线一致。
  - AC-4: 核心约束变更时，必须同步更新 core PRD 与相关模块 PRD。
- Non-Goals:
  - 不在 core PRD 中替代各模块详细技术设计。
  - 不在 core PRD 中定义具体业务数值平衡。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: `rg`/文档治理脚本/测试矩阵文档（`testing-manual.md`）用于规则核验；AI 相关具体策略由业务模块维护。
- Evaluation Strategy: 通过定期抽查 PRD-ID 映射完整性、发布门禁执行一致性、文档治理脚本结果验证执行质量。

## 4. Technical Specifications
- Architecture Overview: core 模块维护仓库级公共约束层，约束对象包括模块文档结构、任务追踪、测试分层与发布门禁。
- Integration Points:
  - `AGENTS.md`
  - `doc/README.md`
  - `testing-manual.md`
  - 各模块 `doc/<module>/prd.md` 与 `doc/<module>/prd.project.md`
- Security & Privacy: core 模块不引入业务数据面，仅约束过程与治理；涉及密钥、签名、隐私数据的要求交由具体模块 PRD 细化。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化跨模块 PRD 结构与追踪规范。
  - v1.1: 引入跨模块变更影响检查清单（设计、代码、测试、发布四维度）。
  - v2.0: 建立仓库级 PRD-ID 到测试产物的自动化追踪报表。
- Technical Risks:
  - 风险-1: 模块边界变化快，core 约束更新滞后。
  - 风险-2: 仅靠人工维护时，PRD-ID 映射存在漏记风险。
