# engineering PRD

## 目标
- 建立 engineering 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 engineering 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 engineering 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/engineering/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/engineering/prd.md`
- 项目管理入口: `doc/engineering/prd.project.md`
- 追踪主键: `PRD-ENGINEERING-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 工程规范分散在多个专题文档，导致文件体量控制、提交门禁、脚本治理与代码质量标准不够统一。
- Proposed Solution: 将 engineering 模块定义为工程治理主文档，统一维护规范、质量门禁、改造节奏与验收口径。
- Success Criteria:
  - SC-1: Rust 单文件超 1200 行新增违规数为 0。
  - SC-2: Markdown 单文件超 500 行新增违规数为 0。
  - SC-3: `scripts/doc-governance-check.sh` 在 required gate 连续通过。
  - SC-4: 工程类任务 100% 映射到 PRD-ENGINEERING-ID。
  - SC-5: `doc/` 根目录与模块根目录平铺文档新增违规数为 0（allowlist 冻结机制）。

## 2. User Experience & Functionality
- User Personas:
  - 工程维护者：需要稳定规则来控制技术债。
  - 贡献开发者：需要清晰门槛和提交前检查路径。
  - 评审者：需要可量化判断变更是否合规。
- User Stories:
  - PRD-ENGINEERING-001: As an 工程维护者, I want enforceable file-size and structure limits, so that maintenance cost stays bounded.
  - PRD-ENGINEERING-002: As a 开发者, I want deterministic pre-commit checks, so that regressions are caught before CI.
  - PRD-ENGINEERING-003: As a 评审者, I want auditable governance evidence, so that review decisions are defensible.
- Acceptance Criteria:
  - AC-1: engineering PRD 明确文件约束、脚本约束、测试分层约束。
  - AC-2: engineering project 文档维护任务拆解与状态。
  - AC-3: 与 `scripts/pre-commit.md`、`testing-manual.md` 的口径一致。
  - AC-4: 每次工程规范变更有对应 devlog 记录。
  - AC-5: 文档治理脚本校验 `doc/.governance/*-allowlist.txt`，可拦截 `doc/*.md` 与 `doc/<module>/*.md` 的非预期新增。
- Non-Goals:
  - 不定义 gameplay/p2p/runtime 业务规则。
  - 不替代模块内部测试策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 文档治理脚本、CI 测试脚本、静态检查脚本。
- Evaluation Strategy: 通过 required/full gate 成功率、违规项统计、回归修复时长衡量工程治理有效性。

## 4. Technical Specifications
- Architecture Overview: engineering 模块聚焦工程流程与规范，不承载业务逻辑；通过脚本与门禁把规范落地到提交链路。
- Integration Points:
  - `scripts/doc-governance-check.sh`
  - `scripts/pre-commit.md`
  - `scripts/fix-precommit.md`
  - `doc/.governance/doc-root-md-allowlist.txt`
  - `doc/.governance/module-root-md-allowlist.txt`
  - `testing-manual.md`
  - `.github/workflows/*`
- Security & Privacy: 仅涉及工程流程元信息；涉及凭据的自动化流程必须遵守最小暴露原则并避免日志泄漏。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化工程规范与门禁指标。
  - v1.1: 补齐高频违规的自动修复建议与脚本化诊断。
  - v2.0: 建立工程规范趋势看板（违规率、修复时长、回归率）。
- Technical Risks:
  - 风险-1: 规范过严导致迭代效率下降。
  - 风险-2: 新脚本引入误报造成 CI 噪声。
