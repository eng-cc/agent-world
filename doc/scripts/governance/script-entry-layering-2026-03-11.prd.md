# Agent World: 脚本分层与主入口 / fallback 入口梳理（2026-03-11）

- 对应设计文档: `doc/scripts/governance/script-entry-layering-2026-03-11.design.md`
- 对应项目管理文档: `doc/scripts/governance/script-entry-layering-2026-03-11.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: `scripts/` 目录已经覆盖构建、测试、发布、站点、Viewer 和 runtime 诊断等多类脚本，但模块主文档尚未给出“哪些是主入口、哪些只在故障或受限环境下作为 fallback 使用”的一览。结果是开发者和 CI 很容易绕过稳定主入口，直接调用低层脚本，造成用法漂移。
- Proposed Solution: 建立脚本分层专题，按 `开发主入口 / CI 发布 / 站点治理 / 长跑回归 / 受控 fallback` 五层给出脚本清单，并为高频入口显式标注推荐主入口与 fallback 入口。
- Success Criteria:
  - SC-1: 至少覆盖当前根 `scripts/` 目录中的高频脚本，并按层归类。
  - SC-2: `capture-viewer-frame.sh` 被明确标注为 fallback，不再与 Web-first 主链路并列。
  - SC-3: `doc/scripts/project.md` 能直接引用本专题完成 `TASK-SCRIPTS-002`。
  - SC-4: 任意常见需求（本地测试、发布门禁、站点巡检、Viewer 诊断）都能反查到一个主入口。

## 2. User Experience & Functionality
- User Personas:
  - 开发者：需要快速找到推荐主入口，而不是自己猜脚本。
  - `qa_engineer`：需要知道哪些脚本属于 required/full 主链路，哪些只是诊断补刀。
  - CI / 发布维护者：需要清楚哪些脚本是流水线稳定接口。
- User Scenarios & Frequency:
  - 日常本地验证：每次开发迭代都可能发生。
  - 发布门禁或回归：每个候选版本都会发生。
  - UI / runtime 故障复现失败：仅在常规 Web-first 链路不能复现时发生。
- User Stories:
  - PRD-SCRIPTS-LAYER-001: As a 开发者, I want a script layering map, so that I can pick the canonical entry point first.
  - PRD-SCRIPTS-LAYER-002: As a `qa_engineer`, I want fallback tools explicitly fenced, so that diagnosis scripts do not replace required test flows.
  - PRD-SCRIPTS-LAYER-003: As a CI maintainer, I want release/test/packaging scripts grouped by responsibility, so that pipeline ownership stays stable.
- Critical User Flows:
  1. `识别需求类型 -> 查分层表 -> 先调用主入口脚本 -> 如失败再判断是否允许 fallback`
  2. `CI / 发布调用脚本 -> 依据脚本层级回溯 owner 与输入输出语义`
  3. `常规链路无法复现图形问题 -> 读取 fallback 条件 -> 才允许调用受控诊断脚本`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 分层清单 | 脚本名、层级、主入口/辅助、用途、典型调用者 | 文档查询后选择脚本 | `unmapped -> mapped -> adopted` | 主入口优先于辅助脚本 | 全员可读，维护者可改 |
| 主入口标注 | 需求类型、推荐脚本、替代脚本 | 默认先走推荐主入口 | `unknown -> canonical` | 一类需求最多 1 个主入口 | `qa_engineer` / CI 维护者可引用 |
| fallback 围栏 | 触发条件、限制、产物要求 | 满足条件才允许调用 fallback | `normal -> fallback_allowed` | fallback 永远排在主入口之后 | 仅诊断/受限环境允许 |
- Acceptance Criteria:
  - AC-1: 专题文档明确列出脚本分层和主入口/ fallback 规则。
  - AC-2: 至少覆盖 `ci-tests.sh`、`release-gate.sh`、`build-game-launcher-bundle.sh`、`run-viewer-web.sh`、`capture-viewer-frame.sh`、`site-link-check.sh` 等高频脚本。
  - AC-3: `doc/scripts/prd.index.md` 与 `doc/scripts/project.md` 回写本专题引用。
  - AC-4: fallback 条件与 `AGENTS.md` 的 Web-first 口径一致。
- Non-Goals:
  - 不在本轮为每个脚本补全参数契约细节。
  - 不修改脚本实现或返回码语义。
  - 不新增自动统计脚本。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 该专题作为 scripts 模块的“入口治理表”，把根 `scripts/` 目录按职责分层，并给出主入口与 fallback 规则，供 `doc/scripts/project.md` 和 testing/manual 引用。
- Integration Points:
  - `scripts/ci-tests.sh`
  - `scripts/release-gate.sh`
  - `scripts/release-prepare-bundle.sh`
  - `scripts/build-game-launcher-bundle.sh`
  - `scripts/run-viewer-web.sh`
  - `scripts/capture-viewer-frame.sh`
  - `scripts/site-link-check.sh`
  - `scripts/site-download-check.sh`
  - `AGENTS.md`
- Edge Cases & Error Handling:
  - 一类需求存在多个候选脚本：文档必须只选一个主入口，其余列为辅助或 fallback。
  - 图形链路不可用：可升级到 `capture-viewer-frame.sh`，但必须先说明 Web-first 不可复现。
  - 历史脚本仍被旧文档引用：先在分层表中登记为“兼容入口 / 待收敛”，不擅自删除。
- Non-Functional Requirements:
  - NFR-SL-1: 分层表必须可被 grep 快速检索。
  - NFR-SL-2: 主入口与 fallback 定义应与 `AGENTS.md`、`testing-manual.md` 一致。
  - NFR-SL-3: 文档更新不得要求同时改动脚本实现。
- Security & Privacy: 仅整理入口语义，不新增任何敏感配置暴露。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (`SL-1`): 完成高频脚本分层与主入口表。
  - v1.1 (`SL-2`): 为高频脚本补参数契约与失败语义。
  - v2.0 (`SL-3`): 建立稳定性趋势指标与治理节奏。
- Technical Risks:
  - 风险-1: 某些脚本职责跨层，主入口选择存在短期争议。
  - 风险-2: 若旧文档继续直接引用低层脚本，主入口治理会再次漂移。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-SCRIPTS-LAYER-001 | `TASK-SCRIPTS-002` / `SL-1` | `test_tier_required` | 抽样检查分层表与高频脚本覆盖 | 开发/CI 找入口效率 |
| PRD-SCRIPTS-LAYER-002 | `TASK-SCRIPTS-002` / `SL-1` | `test_tier_required` | 检查 `capture-viewer-frame.sh` fallback 围栏表述 | Web-first 与诊断链路边界 |
| PRD-SCRIPTS-LAYER-003 | `TASK-SCRIPTS-002` / `SL-1` | `test_tier_required` | 检查 project/index 互链与任务回写 | scripts 模块治理入口一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| `DEC-SL-001` | 先定义入口层级，再补参数契约 | 直接逐脚本补 help 文档 | 没有入口分层时，参数文档也会继续漂移。 |
| `DEC-SL-002` | `capture-viewer-frame.sh` 只作为 fallback | 与 Web-first 链路并列推荐 | 已有工程约束明确 Web-first 优先。 |
| `DEC-SL-003` | 对高频需求只选一个主入口 | 保留多个同级主入口 | 可减少误用与维护分叉。 |
