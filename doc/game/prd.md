# game PRD

审计轮次: 1

## 目标
- 建立 game 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 game 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 game 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/game/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/game/prd.md`
- 项目管理入口: `doc/game/prd.project.md`
- 文件级索引: doc/game/prd.index.md
- 追踪主键: `PRD-GAME-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 玩法规则、经济系统、战争治理和发行可玩性要求分布在多份专题文档，缺少统一入口来描述游戏模块的产品目标与验收指标。
- Proposed Solution: 以 game PRD 作为 gameplay 设计总入口，统一定义核心循环、规则层边界、数值治理和发行质量门槛。
- Success Criteria:
  - SC-1: 新增 gameplay 功能均能映射到 PRD-GAME-ID。
  - SC-2: 核心玩法场景（新手/经济/战争）在测试矩阵中具备对应用例。
  - SC-3: 每次版本发布前至少完成一轮可玩性卡片收集并回填闭环。
  - SC-4: 关键玩法规则变更同步更新 game PRD 与 project 文档。

## 2. User Experience & Functionality
- User Personas:
  - 玩法设计者：需要统一管理玩法目标与平衡约束。
  - 玩法开发者：需要规则层与实现层的映射边界。
  - 发行评审者：需要可度量的可玩性验收标准。
- User Scenarios & Frequency:
  - 玩法规则迭代：每个玩法改动周期至少 1 次规则审阅。
  - 核心循环回归：每周执行，覆盖新手/经济/战争路径。
  - 发布前可玩性评估：每个候选版本至少 1 次。
  - 缺陷复盘与再平衡：高优先级问题关闭前必须复测。
- User Stories:
  - PRD-GAME-001: As a 玩法设计者, I want a canonical gameplay blueprint, so that feature decisions are coherent.
  - PRD-GAME-002: As a 玩法开发者, I want clear rule-layer boundaries, so that runtime and gameplay modules evolve safely.
  - PRD-GAME-003: As a 发行评审者, I want measurable playability gates, so that release readiness is objective.
- Critical User Flows:
  1. Flow-GAME-001: `玩法需求提出 -> 规则层建模 -> 映射实现边界 -> 进入开发`
  2. Flow-GAME-002: `执行核心循环回归 -> 记录可玩性问题 -> 分级 -> 回填修复任务`
  3. Flow-GAME-003: `发布前汇总可玩性证据 -> 对照门禁 -> 输出放行结论`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 核心玩法循环 | 场景、动作、资源、结果 | 执行循环并记录关键指标 | `designed -> implemented -> validated` | 先主循环后扩展循环 | 玩法负责人审核变更 |
| 可玩性问题分级 | 问题描述、严重级、复现步骤、责任人 | 提交后自动进入待修复队列 | `opened -> triaged -> fixed -> verified` | 高严重级优先 | 评审者可调整级别 |
| 发行门禁评审 | 证据包、风险等级、放行建议 | 审查后给出 go/no-go | `pending -> reviewed -> released/blocked` | 风险优先级驱动结论 | 发布负责人最终决策 |
- Acceptance Criteria:
  - AC-1: game PRD 覆盖核心玩法循环、治理机制、测试口径。
  - AC-2: game project 文档任务项可映射到 PRD-GAME-001/002/003。
  - AC-3: 与 `doc/game/gameplay/gameplay-top-level-design.prd.md`、`doc/game/gameplay/gameplay-engineering-architecture.md` 口径一致。
  - AC-4: 发行前可玩性回归必须在 testing 手册与测试结果中可追溯。
- Non-Goals:
  - 不在本 PRD 中给出逐条数值参数表。
  - 不替代 runtime/p2p 的底层实现设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: LLM 行为测试套件、场景驱动回归、可玩性卡片采集流程。
- Evaluation Strategy: 以场景可达成率、关键动作成功率、可玩性反馈缺陷收敛时长作为评估指标。

## 4. Technical Specifications
- Architecture Overview: game 模块定义玩法层抽象，依赖 world-runtime 提供规则执行与资源约束，依赖 world-simulator 与 testing 模块提供可观测与验收。
- Integration Points:
  - `doc/game/gameplay/gameplay-top-level-design.prd.md`
  - `doc/game/gameplay/gameplay-engineering-architecture.md`
  - `doc/playability_test_result/prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 空场景配置：缺少关键玩法配置时禁止进入验收并给出缺失项。
  - 数据异常：数值配置越界时阻断合入并输出规则冲突说明。
  - 权限不足：非玩法负责人不得直接修改核心门禁阈值。
  - 并发冲突：同一玩法规则并行修改时需合并评审再落库。
  - 反馈缺失：无可玩性证据时不得进入发布评审。
  - 回归超时：关键循环回归超时需保留中间产物并重试。
- Non-Functional Requirements:
  - NFR-GAME-1: 关键玩法回归覆盖率 100%（新手/经济/战争）。
  - NFR-GAME-2: 高优先级可玩性问题发布前闭环率 >= 95%。
  - NFR-GAME-3: 玩法门禁结论具备完整证据链（命令/日志/结论）。
  - NFR-GAME-4: 玩法规则口径在模块文档中 1 个工作日内同步。
  - NFR-GAME-5: 玩法改动必须可追溯到 PRD-ID。
- Security & Privacy: gameplay 不直接处理密钥；涉及玩家反馈与行为数据时遵循最小化采集与脱敏记录。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 建立 gameplay 统一设计基线与验收指标。
  - v1.1: 对齐战争/治理/经济三条主循环的跨模块测试门禁。
  - v2.0: 形成玩法改动到可玩性结果的量化闭环报表。
- Technical Risks:
  - 风险-1: 玩法复杂度上升导致规则冲突。
  - 风险-2: 只看技术测试通过而忽略真实可玩性退化。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-GAME-001 | TASK-GAME-001/002/005 | `test_tier_required` | 核心循环验收矩阵检查 | 玩法主循环一致性 |
| PRD-GAME-002 | TASK-GAME-002/003/005 | `test_tier_required` + `test_tier_full` | 规则层边界回归、跨模块联动抽样 | gameplay/runtime 协同稳定性 |
| PRD-GAME-003 | TASK-GAME-003/004/005 | `test_tier_required` | 可玩性证据与发布门禁核验 | 发布质量与玩家体验风险 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-GAME-001 | 以玩法循环为需求主轴组织验收 | 以功能列表平铺验收 | 循环视角更贴近真实体验链路。 |
| DEC-GAME-002 | 引入问题分级与闭环模板 | 缺陷统一平级处理 | 可优化修复优先级与发布节奏。 |
| DEC-GAME-003 | 发布评审绑定可玩性证据 | 仅依赖技术测试 | 能降低“可运行但不好玩”的发布风险。 |
