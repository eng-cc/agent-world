# oasis7: Builtin Wasm Determinism Gate Required Check 保护

- 对应设计文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.design.md`
- 对应项目管理文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: `Wasm Determinism Gate / verify-wasm-determinism (m1|m4|m5)` 若未被固化为 required checks，会导致分支保护配置依赖手工操作并产生漂移风险。
- Proposed Solution: 提供基于 `gh api` 的自动化脚本，增量维护 `main` 分支 required checks，兼容“已保护/未保护”两种状态，并默认对齐 `Wasm Determinism Gate` 的三个 verify job。
- Success Criteria:
  - SC-1: 指定 check 可稳定注入并保留现有 required checks 并集。
  - SC-2: 脚本可在 dry-run 与实际执行模式下输出可审计结果。
  - SC-3: 分支保护配置在重复执行下保持幂等。
  - SC-4: 默认上下文与现行 `.github/workflows/wasm-determinism-gate.yml` 保持一致。

## 2. User Experience & Functionality
- User Personas:
  - 仓库管理员：需要低风险维护 required checks。
  - CI 维护者：需要避免分支保护口径漂移。
  - 审计 / 发布负责人：需要可追溯的保护策略变更记录。
- User Scenarios & Frequency:
  - 新增 required check：workflow 调整后执行一次注入。
  - 审计复核：定期 dry-run 检查保护策略一致性。
  - 保护策略修复：误配置时脚本重放修正。
- User Stories:
  - PRD-TESTING-CI-REQUIRED-001: As a 仓库管理员, I want required check updates automated, so that branch protection remains consistent.
  - PRD-TESTING-CI-REQUIRED-002: As a CI 维护者, I want existing required checks preserved during updates, so that no accidental regression occurs.
  - PRD-TESTING-CI-REQUIRED-003: As a 审计负责人, I want deterministic script output, so that policy changes are traceable.
- Critical User Flows:
  1. Flow-REQUIRED-001: `解析 repo/branch/check 参数 -> 查询保护状态 -> 构造目标 payload`
  2. Flow-REQUIRED-002: `已保护分支 -> patch required_status_checks -> 输出最终 checks`
  3. Flow-REQUIRED-003: `未保护分支 -> 创建最小保护策略 -> 注入 required checks -> 验证生效`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| required check 注入 | repo、branch、check context、strict | 执行 `ci-ensure-required-checks.py` | `planned -> applied -> verified` | 对 checks 取并集去重 | 需 repo 写权限 |
| 分支保护兼容处理 | 保护状态、payload 模式 | 已保护 patch / 未保护 create | `detected -> patched/created` | 保留既有策略优先 | 管理员可执行 |
| 审计输出 | dry-run 结果、最终 checks 列表 | 打印结果并可归档 | `generated -> reviewed` | 输出按 context 排序 | 所有维护者可读 |
- Acceptance Criteria:
  - AC-1: `scripts/ci-ensure-required-checks.py` 支持关键参数。
  - AC-2: 默认注入 `Wasm Determinism Gate / verify-wasm-determinism (m1|m4|m5)` 三个上下文。
  - AC-3: 支持 `--dry-run`，并输出最终生效 checks 列表。
  - AC-4: 已保护与未保护分支场景均可成功处理。
- Non-Goals:
  - 不改造 workflow 内部执行逻辑；仅维护 required check 上下文。
  - 不改造组织级 ruleset。
  - 不替代 testing-manual 的执行说明。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本任务为 GitHub 分支保护自动化）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 使用 `gh api` 读取并更新分支保护配置，通过脚本实现策略幂等与可审计执行。
- Integration Points:
  - `scripts/ci-ensure-required-checks.py`
  - `.github/workflows/wasm-determinism-gate.yml`
  - `scripts/ci-m1-wasm-summary.sh`
  - `scripts/ci-verify-m1-wasm-summaries.py`
  - `scripts/wasm-release-evidence-report.sh`
  - GitHub REST API（branch protection）
- Edge Cases & Error Handling:
  - 权限不足（403）：脚本应明确提示 token scope / 权限问题。
  - payload 不完整（422）：未保护分支创建策略时需补齐最小字段集。
  - context 名称不一致：执行前校验目标 check 名称，避免误配置。
  - 并发修改冲突：重复执行应幂等，不覆盖无关 checks。
- Non-Functional Requirements:
  - NFR-REQUIRED-1: 脚本重复执行结果一致（幂等）。
  - NFR-REQUIRED-2: 保护策略变更可审计（含 dry-run 与实际结果）。
  - NFR-REQUIRED-3: 失败信息具备直接可操作性（权限 / 参数 / context）。
- Security & Privacy: 操作受 GitHub 权限控制，需最小权限 token；日志避免输出敏感 token 信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (M1): 设计文档与项目管理文档落地。
  - v1.1 (M2): 脚本实现与本地语法校验。
  - v2.0 (M3/M4): 仓库应用验证、手册同步与状态收口。
- Technical Risks:
  - 风险-1: 管理员权限不足导致 API 调用失败。
  - 风险-2: 分支未保护场景下 payload 缺字段触发 422。
  - 风险-3: check context 名称漂移导致保护无效。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-CI-REQUIRED-001 | T1/T2/T3 | `test_tier_required` | 脚本参数与 API 调用路径验证 | 分支保护自动化稳定性 |
| PRD-TESTING-CI-REQUIRED-002 | T2/T3 | `test_tier_required` | 已保护/未保护场景并集策略验证 | 既有 required checks 安全性 |
| PRD-TESTING-CI-REQUIRED-003 | T3/T4 | `test_tier_required` | dry-run / 执行输出审计检查 | 治理可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-REQUIRED-001 | 用脚本自动维护 required checks | 手工页面配置 | 自动化可减少漂移并提升可复现性。 |
| DEC-REQUIRED-002 | 采用并集 patch 策略 | 覆盖式重写 required checks | 并集策略可避免误删既有门禁。 |
| DEC-REQUIRED-003 | 默认上下文绑定 `Wasm Determinism Gate` 的三个 verify job | 继续引用历史 multi-runner context | 现行 workflow 已切换，默认上下文必须跟随。 |

## 原文约束点映射（内容保真）
- 原“目标（固化 required check、自动化、并集更新）” -> 仍保留，但默认上下文切换为 `Wasm Determinism Gate`。
- 原“接口/数据（脚本参数、依赖）” -> 保留在当前现行 workflow 与脚本集合内。
