# readme PRD

审计轮次: 4

## 目标
- 建立 readme 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 readme 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 readme 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/readme/project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/readme/prd.md`
- 项目管理入口: `doc/readme/project.md`
- 文件级索引: doc/readme/prd.index.md
- 追踪主键: `PRD-README-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: README 与相关入口文档长期承载架构、运行、规则、发布口径，历史上容易出现口径漂移与链接失效。
- Proposed Solution: 将 readme 模块定义为“对外口径主控层”，统一入口信息、跨文档引用、术语定义与更新策略。
- Success Criteria:
  - SC-1: README 关键章节与模块 PRD 引用一致率达到 100%。
  - SC-2: 对外入口链接有效性检查持续通过。
  - SC-3: 术语与架构描述变更在 1 个工作日内同步到 README 体系。
  - SC-4: readme 相关变更全部具备 PRD-ID 与 devlog 追踪。

## 2. User Experience & Functionality
- User Personas:
  - 新贡献者：需要快速理解系统边界与入口。
  - 外部评审者：需要准确获取当前实现状态与能力。
  - 维护者：需要低成本维护跨文档一致性。
- User Scenarios & Frequency:
  - 新人入项阅读：入项首日必读，建立整体认知。
  - 对外评审准备：每次外部评审前执行入口核对。
  - 文档同步巡检：每周至少 1 次。
  - 发布前口径复核：每个版本候选至少 1 次。
- User Stories:
  - PRD-README-001: As a 新贡献者, I want a reliable top-level narrative, so that onboarding time is reduced.
  - PRD-README-002: As an 评审者, I want consistent architecture statements, so that technical due diligence is faster.
  - PRD-README-003: As a 维护者, I want explicit sync rules, so that docs do not drift.
- PRD-README-004: As a `liveops_community`, I want an external communication brief anchored to internal release evidence, so that public-facing messaging stays consistent with current candidate status and risk boundaries.
- PRD-README-005: As a `liveops_community`, I want a reusable release communication template, so that future candidate briefs follow the same structure, evidence links, and review chain.
- PRD-README-006: As a `liveops_community`, I want an announcement/changelog draft derived from approved messaging, so that formal external copy can start from a safe, audited baseline.
- PRD-README-007: As a `liveops_community`, I want a reusable announcement/changelog template, so that future external drafts follow the same sections, source links, and review states.
- PRD-README-008: As a 仓库访客, I want the root README to reflect the current preview posture, so that I do not mistake the repo for a live release landing page.
- PRD-README-009: As a `producer_system_designer`, I want repo-home copy aligned with site and communication docs, so that public promises stay consistent.
- Critical User Flows:
  1. Flow-RM-001: `阅读 README -> 跳转模块入口 -> 快速定位目标能力`
  2. Flow-RM-002: `检测口径变更 -> 更新入口文档 -> 校验链接 -> 发布同步`
  3. Flow-RM-003: `发布前执行巡检 -> 汇总冲突 -> 修复后复核`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 顶层入口导航 | 模块名称、入口链接、摘要 | 点击跳转模块文档 | `draft -> published -> refreshed` | 入口按模块矩阵排序 | 所有人可读，维护者可更新 |
| 口径一致性同步 | 术语、架构描述、更新时间 | 检测冲突并回写更新 | `detected -> synced -> verified` | 核心入口优先同步 | 文档 owner 审核生效 |
| 链接可用性巡检 | 链接地址、状态、修复建议 | 自动检查并输出报告 | `checked -> broken/fixed` | 断链优先修复 | 维护者可处理 |
- Acceptance Criteria:
  - AC-1: readme PRD 明确入口文档职责边界。
  - AC-2: readme project 文档维护同步任务与状态。
  - AC-3: README 与 `world-rule.md`、`testing-manual.md`、模块 PRD 的链接链路可用。
  - AC-4: 口径更新有明确触发条件与同步时限。
- Non-Goals:
  - 不在 readme PRD 中替代各模块详细设计。
  - 不在 readme PRD 中定义测试用例细节。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 文档链接检查、术语一致性校验、入口巡检脚本。
- Evaluation Strategy: 以链接可用率、口径冲突数、修复时长、评审返工率评估。

## 4. Technical Specifications
- Architecture Overview: readme 模块属于文档入口层，负责跨模块信息汇总、术语统一和导航稳定性。
- Integration Points:
  - `README.md`
  - `world-rule.md`
  - `testing-manual.md`
  - `doc/README.md`
- Edge Cases & Error Handling:
  - 链接失效：断链必须在巡检报告中暴露并进入修复队列。
  - 口径冲突：冲突出现时禁止发布“已同步”状态。
  - 空入口：模块入口缺失时标记高优告警并补齐跳转。
  - 权限不足：非维护者不得直接修改对外核心描述。
  - 并发编辑：同文件并发更新时需合并后重跑链接检查。
  - 历史重定向：legacy redirect 必须保留指向并声明主入口。
- Non-Functional Requirements:
  - NFR-RM-1: 顶层入口链接可用率 100%。
  - NFR-RM-2: 术语冲突修复 SLA <= 1 个工作日。
  - NFR-RM-3: README 与模块 PRD 关键引用一致率 100%。
  - NFR-RM-4: 发布前口径巡检覆盖率 100%。
  - NFR-RM-5: 对外文档不得暴露敏感配置信息。
- Security & Privacy: 对外文档不得暴露敏感配置与密钥信息；示例配置需使用脱敏样例。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 README 入口职责与同步流程。
  - v1.1: 增加自动化链接/术语巡检任务。
  - v2.0: 建立入口文档质量趋势指标（漂移率、修复时长）。
- Technical Risks:
  - 风险-1: 高频变更导致跨文档同步延迟。
  - 风险-2: 大范围重构时导航信息失真。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-README-001 | TASK-README-001/002/005 | `test_tier_required` | 入口结构与导航可达检查 | 新人入项与外部阅读体验 |
| PRD-README-002 | TASK-README-002/003/005 | `test_tier_required` | 架构口径一致性与链接巡检 | 技术评审效率与准确性 |
| PRD-README-003 | TASK-README-003/004/005 | `test_tier_required` | 同步流程与修复节奏抽样复核 | 文档长期稳定性 |
| PRD-README-004 | TASK-README-006 | `test_tier_required` | 对外口径简报、禁用表述与回滚口径抽样复核 | 版本候选外部沟通一致性 |
| PRD-README-005 | TASK-README-007 | `test_tier_required` | 对外口径模板、evidence-link 字段与审批链抽样复核 | 版本候选口径模板复用性 |
| PRD-README-006 | TASK-README-008 | `test_tier_required` | 公告 / changelog 底稿、draft 状态与 FAQ 结构抽样复核 | 对外发布底稿一致性 |
| PRD-README-007 | TASK-README-009 | `test_tier_required` | 公告模板、source links 与 review status 抽样复核 | 公告底稿模板复用性 |
| PRD-README-008 | TASK-README-010 | `test_tier_required` | 根 README 状态段含技术预览 / 尚不可玩 / 公告准备态 | 仓库首页状态理解 |
| PRD-README-009 | TASK-README-010 | `test_tier_required` | README 与 site / brief 口径一致 | 公开口径一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-RM-001 | README 作为对外入口主控层 | 各模块对外独立叙述 | 容易产生口径漂移。 |
| DEC-RM-002 | 入口链接定期自动巡检 | 仅人工抽查 | 自动巡检可降低断链遗漏率。 |
| DEC-RM-003 | 口径更新设置同步时限 | 无明确时限 | 时限可提高协作可预测性。 |
| DEC-RM-004 | 候选级对外说明先走简报专题，再决定是否升级到 README / 公告 | 直接在 README 或外部公告里写最终口径 | 先用简报控边界，能避免对外承诺过早固化。 |
| DEC-RM-005 | 用模板沉淀 release communication 结构 | 每次按临时文案自由发挥 | 模板能提高后续候选的复用率与审阅效率。 |
| DEC-RM-006 | 在简报之后先落 announcement/changelog draft，再决定是否正式发布 | 直接将简报对外公开 | 底稿更接近外部文风，同时仍保留 draft 审核缓冲层。 |
| DEC-RM-007 | 将 announcement/changelog 底稿继续模板化 | 每个候选手写公告底稿 | 模板化能降低重写成本，并稳定审核结构。 |
| DEC-RM-008 | 根 README 只对齐状态段，不重写整份首页 | 为修正口径重做全部 README 文案 | 最小变更即可消除仓库首页与 site 的状态分叉。 |
