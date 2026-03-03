# testing PRD

## 目标
- 建立 testing 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 testing 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 testing 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/testing/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/testing/prd.md`
- 项目管理入口: `doc/testing/prd.project.md`
- 追踪主键: `PRD-TESTING-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 测试套件覆盖范围广（required/full、Web 闭环、长跑、分布式），但目标与触发矩阵若不集中维护，容易出现“通过 CI 但缺少真实风险覆盖”。
- Proposed Solution: 以 testing PRD 统一定义分层测试体系、触发条件、证据标准与发布门禁，并对齐 `testing-manual.md`。
- Success Criteria:
  - SC-1: 关键改动路径均可映射到明确测试层级（S0~S10）。
  - SC-2: required/full 门禁持续可用且与手册口径一致。
  - SC-3: Web UI 闭环与分布式长跑在发布流程中有可追溯证据。
  - SC-4: 测试任务 100% 映射 PRD-TESTING-ID。
  - SC-5: 活跃 testing 专题文档按批次完成人工迁移到 strict schema，并统一 `*.prd.md` / `*.prd.project.md` 命名。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：需要统一分层模型与执行标准。
  - 功能开发者：需要明确改动后最小必跑集合。
  - 发布负责人：需要审计级测试证据判断放行。
- User Scenarios & Frequency:
  - 开发分支回归：每次核心改动后触发一次 required 路径。
  - 发布候选验证：每个候选版本执行 required + full 组合。
  - 专项长跑：高风险链路按周执行并沉淀趋势结果。
  - 失效复盘：出现逃逸缺陷后补齐回归与触发矩阵。
- User Stories:
  - PRD-TESTING-001: As a 测试维护者, I want one canonical testing strategy, so that suite evolution stays coherent.
  - PRD-TESTING-002: As a 开发者, I want clear trigger matrices, so that I can run the right tests efficiently.
  - PRD-TESTING-003: As a 发布负责人, I want auditable evidence bundles, so that release decisions are defensible.
  - PRD-TESTING-004: As a 文档维护者, I want each legacy testing topic doc manually migrated with content-preserving rewrite, so that historical intent remains accurate after format upgrade.
- Critical User Flows:
  1. Flow-TST-001: `识别改动类型 -> 匹配 S0~S10 -> 执行 required -> 输出结果`
  2. Flow-TST-002: `发布前执行 full 套件 -> 汇总命令/日志/截图 -> 生成证据包`
  3. Flow-TST-003: `线上问题复盘 -> 回填触发矩阵 -> 增加回归用例 -> 验证闭环`
  4. Flow-TST-004: `逐篇阅读 legacy 专题文档 -> 按 strict schema 人工重写 -> 改名为 .prd/.prd.project -> 回归校验`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 分层测试触发 | 改动类型、测试层级、命令集合 | 依据矩阵选择最小必跑集合 | `planned -> running -> passed/failed` | 默认 required 优先，发布加跑 full | 开发者可执行，发布者可放行 |
| 证据包归档 | 命令、日志、截图、结论、责任人 | 执行后归档并建立索引 | `collecting -> archived -> reviewed` | 按版本与模块分层索引 | 测试维护者负责最终校验 |
| 缺陷回归闭环 | 缺陷ID、触发条件、修复提交、复测结论 | 缺陷关闭前必须绑定回归记录 | `opened -> fixed -> regressed -> closed` | 高风险缺陷优先回归 | QA/维护者可更新状态 |
| 文档格式迁移 | 旧文档路径、约束点清单、目标命名 | 人工重写并更名，补全映射与验证证据 | `inventory -> migrated -> validated` | 先迁移活跃文档、后迁移归档文档 | 维护者审批迁移质量，贡献者执行 |
- Acceptance Criteria:
  - AC-1: testing PRD 覆盖分层模型、触发矩阵、证据规范。
  - AC-2: testing project 文档维护分层测试演进任务。
  - AC-3: 与 `testing-manual.md` 保持一致且互相引用。
  - AC-4: 新增测试流程需标注 `test_tier_required` 或 `test_tier_full`。
  - AC-5: 每个迁移批次必须提供“原文约束点 -> 新章节映射”并通过文档治理检查。
- Non-Goals:
  - 不在本 PRD 中替代业务模块的功能设计。
  - 不承诺所有测试都进入 CI 默认路径。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: `scripts/ci-tests.sh`、Playwright 闭环工具、长跑脚本、结果汇总工具。
- Evaluation Strategy: 通过门禁通过率、缺陷逃逸率、回归定位时长、证据完整度衡量测试体系质量。

## 4. Technical Specifications
- Architecture Overview: testing 模块是仓库级验证层，负责连接代码改动、测试触发、证据产物与发布门禁。
- Integration Points:
  - `testing-manual.md`
  - `doc/testing/manual/web-ui-playwright-closure-manual.md`
  - `scripts/ci-tests.sh`
  - `.github/workflows/*`
- Edge Cases & Error Handling:
  - 网络波动：外部依赖失败时记录失败签名并支持重试，不静默跳过。
  - 空产物：测试通过但缺证据产物视为不通过。
  - 权限不足：CI 环境权限不足时标记阻塞并输出最小修复建议。
  - 超时：长跑套件超时需产出中间状态，防止误判为无结果。
  - 并发冲突：同一产物路径并发写入时强制分目录隔离。
  - 数据异常：日志格式破损时保留原始文件并标记解析失败。
  - 迁移断链：文档改名后若引用未同步，需在同批次修复并复测。
- Non-Functional Requirements:
  - NFR-TST-1: required 套件变更前后执行时间波动 <= 20%。
  - NFR-TST-2: 发布证据包字段完整率 100%。
  - NFR-TST-3: 关键链路缺陷逃逸率持续下降（按月跟踪）。
  - NFR-TST-4: 测试手册与脚本口径冲突数为 0。
  - NFR-TST-5: 测试执行结果可在 30 分钟内完成追溯定位。
  - NFR-TST-6: 文档迁移批次在不降低治理质量的前提下保持可审阅粒度（每任务对应单文档或单专题）。
- Security & Privacy: 测试日志与产物需避免泄露凭据；外部 API 测试使用最小化数据并执行脱敏。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 testing 分层模型与证据标准。
  - v1.1: 补齐高风险路径的触发矩阵自动检查。
  - v2.0: 建立跨版本测试质量趋势分析与发布建议。
- Technical Risks:
  - 风险-1: 套件增长导致执行成本上升。
  - 风险-2: 手册与脚本不一致导致执行偏差。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-001 | TASK-TESTING-001/002/005/006 | `test_tier_required` | S0~S10 触发矩阵核验、手册一致性检查 | 分层测试入口与执行标准 |
| PRD-TESTING-002 | TASK-TESTING-002/003/006 | `test_tier_required` + `test_tier_full` | 证据模板抽样、发布前必填字段检查 | 发布链路可信性与可复现性 |
| PRD-TESTING-003 | TASK-TESTING-003/004/006 | `test_tier_full` | 趋势指标回顾、缺陷逃逸复盘 | 长期质量治理与发布风险控制 |
| PRD-TESTING-004 | TASK-TESTING-007/008/009/010/011/012/013/014/015/016 | `test_tier_required` | 原文约束点映射审查、命名与引用回归检查 | 专题文档可维护性与追溯一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-TST-001 | 采用 required/full 分层验证 | 全量套件每次必跑 | 保持效率与覆盖平衡。 |
| DEC-TST-002 | 证据包作为发布必备输入 | 只记录口头结论 | 审计与追溯能力不足风险更高。 |
| DEC-TST-003 | 以手册驱动触发矩阵统一口径 | 各模块自行定义测试口径 | 可减少跨模块冲突和遗漏。 |
| DEC-TST-004 | legacy 专题文档采用逐篇人工迁移并统一 `.prd` 命名 | 自动脚本批量改写 | 可确保内容语义与约束不丢失。 |
