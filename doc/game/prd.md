# game PRD

审计轮次: 4

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
  - SC-5: 微循环关键动作具备可见反馈与计时提示，发布前可玩性卡片评分显著提升。
  - SC-6: 长期在线场景下，治理改动与世界状态具备 tick 级可验证证书和可重放一致性证明。

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
  - PRD-GAME-004: As a 玩家/评测者, I want micro-loop feedback visibility, so that control and pacing are reliable.
  - PRD-GAME-005: As a 运行治理者, I want deterministic distributed execution and governance guardrails, so that the world can run online for the long term.
- Critical User Flows:
  1. Flow-GAME-001: `玩法需求提出 -> 规则层建模 -> 映射实现边界 -> 进入开发`
  2. Flow-GAME-002: `执行核心循环回归 -> 记录可玩性问题 -> 分级 -> 回填修复任务`
  3. Flow-GAME-003: `发布前汇总可玩性证据 -> 对照门禁 -> 输出放行结论`
  4. Flow-GAME-004: `治理提案 -> 投票 -> timelock -> epoch 生效 -> tick 证书审计回放`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 核心玩法循环 | 场景、动作、资源、结果 | 执行循环并记录关键指标 | `designed -> implemented -> validated` | 先主循环后扩展循环 | 玩法负责人审核变更 |
| 可玩性问题分级 | 问题描述、严重级、复现步骤、责任人 | 提交后自动进入待修复队列 | `opened -> triaged -> fixed -> verified` | 高严重级优先 | 评审者可调整级别 |
| 发行门禁评审 | 证据包、风险等级、放行建议 | 审查后给出 go/no-go | `pending -> reviewed -> released/blocked` | 风险优先级驱动结论 | 发布负责人最终决策 |
| 分布式执行与治理 | `tick/block hash`、`state_root`、治理提案元数据、身份信誉/抵押 | 发起提案、投票、队列生效、紧急刹车/否决 | `draft -> voting -> queued -> applied/rejected` | tick 全序执行 + epoch 边界生效 | 治理角色+阈值双重校验 |
- 核心玩法循环验收矩阵（TASK-GAME-002）:
| 循环 | 验收场景（Given / When / Then） | 规则层边界（PRD-GAME-002） | 证据事件/状态 | `test_tier_required` 入口 | 通过阈值（Done） | 失败处置 |
| --- | --- | --- | --- | --- | --- | --- |
| 新手循环（前 1~3 天） | Given `llm_bootstrap` 场景可启动；When 玩家完成“选定目标 -> 首次指令 -> 收到反馈”；Then 在单次会话内形成 `观察 -> 决策 -> 反馈 -> 调整` 闭环。 | 新手引导只消费 runtime 已开放动作（不允许越权改写世界状态）；动作被拒绝时必须返回可解释原因。 | `DomainEvent::ActionAccepted`；viewer 任务循环快照与倒计时提示。 | `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::gameplay_protocol::gameplay_actions_emit_action_accepted_before_resolution_event -- --nocapture`；`env -u RUSTC_WRAPPER cargo test -p agent_world_viewer player_mission_tests:: -- --nocapture` | 必须出现“先接受后解析”的动作证据；玩家任务循环快照能稳定展示剩余提示与反馈计时。 | 阻断合入；补齐失败动作链路日志与 UI 快照，按 P1 建立修复任务并复测。 |
| 经济循环 | Given 双方具备可结算资源；When 执行 `Open -> Accept -> Settle` 经济合约；Then 合约状态与声誉/税费变化可回放。 | 经济规则不得绕过资源守恒；结算溢出/配额/黑名单冲突必须原子拒绝且不污染状态。 | `DomainEvent::EconomicContractOpened/Accepted/Settled/Expired`；`economic_contracts` 状态与声誉快照。 | `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::gameplay_protocol::economic_contract_ -- --nocapture` | 合约终态必须可解释（`Settled` 或 `Expired`）；税费、信誉奖励与策略上限一致；异常路径无半提交状态。 | 阻断合入；输出冲突合约 ID、策略参数与状态差异，回归通过前不得进入发布评审。 |
| 战争循环 | Given 至少两联盟且满足动员成本；When 发起宣战并推进 tick；Then 战争按时结算并写入胜负与参与者后果。 | 宣战必须校验联盟成员身份与动员资源；活动战争期间违反约束的动作（如违规加入）必须拒绝。 | `DomainEvent::WarDeclared/WarConcluded`；`wars` 状态（`active/winner/loser/concluded_at`）。 | `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::gameplay_protocol::war_ -- --nocapture` | 战争必须在设计时长内自动收敛；胜负、资源后果与事件链一致；拒绝路径具备明确规则原因。 | 阻断合入；保存冲突 tick 与战报证据，按 P0 进入规则修复并执行全链路复测。 |
- 矩阵基线一致性校验：`env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required scenario_specs_match_ids -- --nocapture`，用于确保场景入口与矩阵引用保持一致。
- 可玩性问题分级与修复闭环模板（TASK-GAME-003）:
| 等级 | 判定条件 | 典型影响 | 发布门禁动作 | 修复时限（SLO） |
| --- | --- | --- | --- | --- |
| P0 | 关键循环不可达、规则越权、同输入不同结果、核心反馈缺失导致“不可玩”。 | 新手/经济/战争任一主循环中断或产生不可恢复分叉。 | 直接 `blocked`，禁止发布；必须完成复测并由发布负责人确认。 | `<= 24h` 完成修复与复测结论。 |
| P1 | 主循环可运行但体验明显退化，存在稳定复现路径且影响关键指标。 | 玩家可继续游玩但决策反馈延迟、收益失真或冲突结局异常。 | 默认阻断；若需放行必须登记风险豁免 ID 与回滚预案。 | `<= 72h` 完成修复，或进入带风险放行审批。 |
| P2 | 体验瑕疵或低频异常，不破坏主循环可达性。 | 文案、引导节奏、次要可见性偏差。 | 可带缺陷放行，但必须进入下一版本回归清单。 | 下一个迭代周期前关闭。 |
| P3 | 观察项或优化建议，暂无稳定复现与用户影响证据。 | 研发/评测发现的潜在改进点。 | 不阻断发布，纳入趋势看板跟踪。 | 按周评审并决定是否升级优先级。 |
- 闭环执行模板（字段 + 流程）:
| 阶段 | 必填字段 | 执行动作 | 状态流转 | 验证与证据 | 权限/时限 |
| --- | --- | --- | --- | --- | --- |
| 问题提报 | `issue_id`、循环类型（新手/经济/战争）、复现步骤、证据路径、`PRD-GAME-ID` | 创建标准卡片并绑定对应循环与版本。 | `opened` | 至少 1 条可复现证据（事件/日志/UI 截图）。 | 评测者可创建；当日完成。 |
| 问题分级 | 严重级、影响范围、责任人、目标修复版本 | 按分级矩阵打标，确认是否触发发布阻断。 | `opened -> triaged` | 关联门禁条目与预计回归入口。 | 玩法负责人批准；`<= 24h`。 |
| 修复执行 | 根因、修复提交、测试计划、回滚方案 | 开发修复并同步 PRD/project 追踪关系。 | `triaged -> fixing` | 提交记录 + 定向测试计划。 | 责任开发执行；P0/P1 按 SLO。 |
| 修复验证 | 回归命令、结果、剩余风险、复测结论 | 执行定向回归与抽样联动回归。 | `fixing -> verified` | 测试日志 + 关键事件/状态对照。 | QA/评审者确认；未通过不得关闭。 |
| 发布结论 | 发布候选版本、豁免单（若有）、审计人 | 形成 go/no-go 结论并回填证据包。 | `verified -> closed` 或 `verified -> deferred` | 发布记录、豁免审批、回滚预案。 | 发布负责人；P0 不允许 `deferred`。 |
- 闭环强制约束:
  - 无复现证据的 P0/P1 不得降级。
  - P0/P1 在 `verified` 前不得进入发布评审。
  - `deferred` 必须带豁免 ID、风险说明、下一次复测日期。
- Acceptance Criteria:
  - AC-1: game PRD 覆盖核心玩法循环、治理机制、测试口径。
  - AC-2: game project 文档任务项可映射到 PRD-GAME-001/002/003。
  - AC-3: 与 `doc/game/gameplay/gameplay-top-level-design.prd.md`、`doc/game/gameplay/gameplay-engineering-architecture.md` 口径一致。
  - AC-4: 发行前可玩性回归必须在 testing 手册与测试结果中可追溯。
  - AC-5: 微循环反馈优化 PRD 定义可见反馈与计时规则，并形成可验证的评分提升目标。
  - AC-6: 新增长期在线分布式专题 PRD，明确 RSM、治理时延生效、身份与惩罚的验收约束。
  - AC-7: 新手/经济/战争三循环均具备 Given/When/Then、规则层边界、证据事件、`test_tier_required` 命令与失败处置，且可直接用于周回归。
  - AC-8: 可玩性问题分级模板覆盖 `P0~P3` 判定、发布阻断规则、责任人和时限，并能直接驱动 `opened -> triaged -> fixing -> verified -> closed/deferred` 闭环。
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
  - `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
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
  - 状态分叉：出现同 tick 不同 `state_root` 时阻断提交并触发恢复流程。
  - 提前生效：治理提案在 `timelock/epoch` 约束前申请生效必须拒绝。
  - 女巫攻击：疑似多号协同投票触发权重冻结与人工复核。
- Non-Functional Requirements:
  - NFR-GAME-1: 关键玩法回归覆盖率 100%（新手/经济/战争）。
  - NFR-GAME-2: 高优先级可玩性问题发布前闭环率 >= 95%。
  - NFR-GAME-3: 玩法门禁结论具备完整证据链（命令/日志/结论）。
  - NFR-GAME-4: 玩法规则口径在模块文档中 1 个工作日内同步。
  - NFR-GAME-5: 玩法改动必须可追溯到 PRD-ID。
  - NFR-GAME-6: RSM 回放一致性偏差率为 0（同输入同版本）。
  - NFR-GAME-7: 治理规则变更 100% 走提案链路并满足 `timelock + epoch` 生效。
  - NFR-GAME-8: 紧急权限触发事件 100% 具备可验签证据和审计记录。
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
| PRD-GAME-003 | TASK-GAME-003/004/005 | `test_tier_required` | 问题分级模板抽样、修复闭环记录核验、发布门禁对账 | 发布质量与玩家体验风险 |
| PRD-GAME-004 | TASK-GAME-006 + TASK-GAMEPLAY-MLF-001/002/003/004 | `test_tier_required` | 微循环反馈可见性回归 + 可玩性卡片评分复核 | 玩家控制感与节奏体验 |
| PRD-GAME-005 | TASK-GAME-008 + TASK-GAME-DCG-001/002/003/004/005/006/007/008/009/010 | `test_tier_required` + `test_tier_full` | Tick 证书、治理时序、身份惩罚闭环验证 | 长期在线一致性与治理安全 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-GAME-001 | 以玩法循环为需求主轴组织验收 | 以功能列表平铺验收 | 循环视角更贴近真实体验链路。 |
| DEC-GAME-002 | 引入问题分级与闭环模板 | 缺陷统一平级处理 | 可优化修复优先级与发布节奏。 |
| DEC-GAME-003 | 发布评审绑定可玩性证据 | 仅依赖技术测试 | 能降低“可运行但不好玩”的发布风险。 |
| DEC-GAME-004 | 以“新手/经济/战争”分循环验收矩阵驱动 `TASK-GAME-002` | 仅保留统一 required/full 命令清单 | 分循环矩阵更易映射规则边界、失败处置与责任归属。 |
| DEC-GAME-005 | 采用 `P0~P3 + 闭环模板 + deferred 豁免` 的分级机制 | 仅保留缺陷列表，不定义状态与门禁 | 可保证问题优先级、修复责任与发布决策可审计。 |
