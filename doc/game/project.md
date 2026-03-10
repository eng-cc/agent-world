# game PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-GAME-001 (PRD-GAME-001) [test_tier_required]: 完成 game PRD 改写，建立玩法设计总入口。
- [x] TASK-GAME-002 (PRD-GAME-001/002) [test_tier_required]: 补齐核心玩法循环（新手/经济/战争）验收矩阵。
- [x] TASK-GAME-003 (PRD-GAME-002/003) [test_tier_required]: 建立可玩性问题分级与修复闭环模板。
- [x] TASK-GAME-004 (PRD-GAME-003) [test_tier_required]: 对接发布前可玩性门禁与回归节奏。
- [x] TASK-GAME-005 (PRD-GAME-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-GAME-006 (PRD-GAME-004) [test_tier_required]: 新增微循环反馈可见性 PRD 与项目文档，完成文档树挂载。
- [x] TASK-GAME-007 (PRD-GAME-004) [test_tier_required]: 落地 runtime 协议与 viewer 反馈闭环并完成回归验证（子任务 `TASK-GAMEPLAY-MLF-001/002/003/004` 已全部完成，见 `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.project.md`）。
- [x] TASK-GAME-008 (PRD-GAME-005) [test_tier_required]: 新增“分布式执行共识/治理共识/身份与反女巫”专题 PRD 与项目管理文档，完成根文档追踪映射。
- [x] TASK-GAME-009 (PRD-GAME-005) [test_tier_required]: 落地 tick 证书链与 `state_root/events_hash` 一致性校验实现（含 replay/save-load 闭环）。
- [x] TASK-GAME-010 (PRD-GAME-005) [test_tier_required]: 落地治理 `timelock + epoch` 生效门禁与紧急刹车/否决约束。
- [x] TASK-GAME-011 (PRD-GAME-005) [test_tier_required + test_tier_full]: 落地身份信誉/抵押权重、女巫检测与惩罚申诉闭环。
- [x] TASK-GAME-012 (PRD-GAME-006) [test_tier_required]: 新增长期在线 P0 生产硬化专题 PRD 与项目管理文档，完成根文档追踪映射。
- [x] TASK-GAME-013 (PRD-GAME-006) [test_tier_required]: 落地状态权威分层（传播层/裁决层）与冲突仲裁拒绝路径。
- [x] TASK-GAME-014 (PRD-GAME-006) [test_tier_required + test_tier_full]: 补齐确定性回放 + 快照回滚 runbook 与演练门禁。
- [x] TASK-GAME-015 (PRD-GAME-006) [test_tier_required + test_tier_full]: 落地反作弊/反女巫对抗检测、惩罚与申诉证据链强化。
- [x] TASK-GAME-016 (PRD-GAME-006) [test_tier_required]: 建立经济源汇审计与通胀/套利告警阈值门禁。
- [x] TASK-GAME-017 (PRD-GAME-006) [test_tier_required]: 补齐可运维性能力（SLO、告警、灰度、灾备演练）与发布阻断规则。
- [ ] TASK-GAME-018 (PRD-GAME-004) [test_tier_required]: 执行微循环可玩性视觉优化二期（控制结果显著化、玩家模式减负、世界可读性增强）并以手动截图闭环验收（见 `TASK-GAMEPLAY-MLF-005/006/007/008`）。

## 依赖
- 模块设计总览：`doc/game/design.md`
- doc/game/prd.index.md
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-engineering-architecture.md`
- `doc/playability_test_result/prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-10
- 当前状态: active
- 下一任务: TASK-GAME-018（微循环可玩性视觉优化二期）
- 阶段收口优先级: `P0`
- 阶段 owner: `viewer_engineer`（发起/裁剪：`producer_system_designer`；验证：`qa_engineer`）
- 阻断条件: 在 `TASK-GAME-018` 完成截图闭环前，当前版本不得以“玩法体验已收口”为前提给出发布 `go` 结论。
- 承接约束: `TASK-GAMEPLAY-MLF-005/006/007/008` 必须统一回写到同一轮视觉优化证据包，并同步引用 `playability_test_result` 模块的反馈口径。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- ROUND-002 进展: gameplay 子簇主从化完成，`TASK-GAMEPLAY-MLF-001/002/003/004` 与 `TASK-GAME-007` 已闭环；分布式长期在线专题已完成设计建档（`TASK-GAME-008`）与执行共识首个实现切片（`TASK-GAME-009`）。
- ROUND-003 进展: `TASK-GAME-010` 已完成，治理 `Queued + timelock/epoch` 门禁与紧急控制（刹车/否决）状态机已落地并通过定向回归。
- ROUND-004 进展: `TASK-GAME-011` 已完成，`TASK-GAME-DCG-007/008`（身份权重快照 + 女巫惩罚申诉闭环）已落地并通过治理/协议/持久化/审计回归。
- ROUND-005 进展: `TASK-GAME-DCG-009` 已完成，P2P 长稳脚本新增共识哈希一致性门禁并通过 triad+chaos 烟测。
- ROUND-006 进展: `TASK-GAME-DCG-010` 已完成，发布门禁报告与回滚预案已输出（含 `soak_release` 基线证据）。
- ROUND-007 进展: `TASK-GAME-002` 已完成，根 PRD 新增“新手/经济/战争”三循环验收矩阵（含 Given/When/Then、规则边界、证据事件、`test_tier_required` 入口与失败处置）。
- ROUND-008 进展: `TASK-GAME-003` 已完成，根 PRD 新增 `P0~P3` 分级标准、`opened -> triaged -> fixing -> verified -> closed/deferred` 闭环模板与强制约束（含豁免门禁）。
- ROUND-009 进展: `TASK-GAME-004` 已完成，根 PRD 新增 `D/RC/D-1/D0` 发布门禁节奏与证据包字段，形成可直接执行的 go/no-go 流程。
- ROUND-010 进展: `TASK-GAME-012` 已完成，新增 `PRD-GAME-006` 长期在线 P0 生产硬化专题（状态权威分层、回放回滚、反作弊、经济闭环、可运维性）并挂载到文档树。
- ROUND-011 进展: `TASK-GAME-013` 已完成，runtime 新增 `authority_source/submission_role` 共识提交分层与冲突拒绝审计事件，完成状态权威分层首个实现切片。
- ROUND-012 进展: `TASK-GAME-014` 已完成，新增 `first_tick_consensus_drift` 漂移定位与 `rollback_to_snapshot_with_reconciliation` 回滚后自动对账能力，并补齐 runbook + 演练门禁。
- ROUND-013 进展: `TASK-GAME-015` 已完成，治理惩罚记录新增 `detection_* + evidence_chain_hash` 证据链字段，惩罚事件指纹防重放与误伤率监控快照能力已落地并通过定向回归。
- ROUND-014 进展: `TASK-GAME-016` 已完成，main token 新增经济源汇审计报表与通胀/套利阈值 gate（`main_token_economy_audit_report/enforce_main_token_economy_gate`），并通过定向回归。
- ROUND-015 进展: `TASK-GAME-017` 已完成，新增 long-run 可运维发布门禁模型（SLO/告警/灰度/灾备 + 经济告警联动）与阻断接口 `enforce_longrun_operability_release_gate`。
- ROUND-016 进展: `TASK-GAME-018` 已立项，进入 viewer 体验层改造与手动截图验收阶段。
- ROUND-017 进展: `TASK-GAME-018` 已进入执行中，`TASK-GAMEPLAY-MLF-005/006` 已完成（控制结果显著条 + 玩家模式默认减负）；`TASK-GAMEPLAY-MLF-008` 已完成一轮 runtime_live 节奏修正并通过手动截图验收（播放稳态由 `+11~12 tick/s` 降至 `+1 tick/s`，暂停 `+0 tick/s`），后续继续收口 `TASK-GAMEPLAY-MLF-007`。
- 说明: 本文档仅维护 game 设计执行状态；过程记录在 `doc/devlog/2026-03-05.md`、`doc/devlog/2026-03-06.md` 与 `doc/devlog/2026-03-07.md`。

## 阶段收口角色交接
## Meta
- Handoff ID: `HO-CORE-20260310-GAME-001`
- Date: `2026-03-10`
- From Role: `producer_system_designer`
- To Role: `viewer_engineer`
- Related Module: `game`
- Related PRD-ID: `PRD-GAME-004`
- Related Task ID: `TASK-GAME-018`
- Priority: `P0`
- Expected ETA: `待接收方确认`

## Objective
- 目标描述：完成微循环可玩性视觉优化二期，并产出能直接进入发布评审的截图闭环证据。
- 成功标准：控制结果显著化、玩家模式减负、世界可读性增强三项同时完成，且 `qa_engineer` 可据此复核。
- 非目标：不在本轮新增 launcher / explorer 体验功能，不扩展与微循环无关的 Viewer 大改。

## Current State
- 当前实现 / 文档状态：`TASK-GAME-018` 为模块唯一未完成主任务，当前已完成 `TASK-GAMEPLAY-MLF-005/006` 与一轮 `TASK-GAMEPLAY-MLF-008` 收口。
- 已确认事实：core 阶段收口将玩法微循环列为 `P0`，未完成前不得给出玩法收口 `go` 结论。
- 待确认假设：`TASK-GAMEPLAY-MLF-007` 是否仍需额外 Viewer 表达改造才能达成“世界可读性增强”。
- 当前失败信号 / 用户反馈：当前项目仍偏“能展示”，需要把“更好玩”变成明确证据。

## Scope
- In Scope: `TASK-GAME-018`、`TASK-GAMEPLAY-MLF-005/006/007/008`、截图 / 视频 / 结论证据回写。
- Out of Scope: 新玩法分支、新区块链浏览器功能、与微循环无关的全局 UI 重构。

## Inputs
- 关键文件：`doc/game/project.md`、`doc/game/prd.md`、相关 `gameplay-micro-loop-*` 专题文档。
- 关键命令：沿用现有 Viewer / playability 截图闭环命令与手动验收流程。
- 上游依赖：`producer_system_designer` 已在 `core` 层确定该项为 `P0`；`qa_engineer` 后续复核证据。
- 现有测试 / 证据：现有手动截图验收记录与 `runtime_live` 节奏修正结果。

## Requested Work
- 工作项 1：收口 `TASK-GAMEPLAY-MLF-007`，完成世界可读性增强。
- 工作项 2：复核并统一 `TASK-GAMEPLAY-MLF-005/006/008` 证据格式。
- 工作项 3：产出可直接进入发布评审的截图 / 视频 / 结论证据包，并回写项目状态。

## Expected Outputs
- 代码改动：如需，仅限支撑 `TASK-GAME-018` 的 Viewer 表达层改动。
- 文档回写：`doc/game/project.md`、必要时相关专题 `project/prd`。
- 测试记录：至少补齐 `test_tier_required` 的截图闭环与结论。
- devlog 记录：在 `doc/devlog/YYYY-MM-DD.md` 中记载结果与遗留项。

## Done Definition
- [ ] 输出满足目标与成功标准
- [ ] 影响面已核对 `producer_system_designer` / `qa_engineer`
- [ ] 对应 `prd.md` / `project.md` 已回写
- [ ] 对应 `doc/devlog/YYYY-MM-DD.md` 已记录
- [ ] required 证据已补齐

## Risks / Decisions
- 已知风险：如果只做视觉 polish 而不统一证据格式，玩法收口仍无法进入 go/no-go 评审。
- 待拍板事项：是否需要把 `TASK-GAMEPLAY-MLF-007` 进一步拆小给 `viewer_engineer`。
- 建议决策：先以最小体验闭环完成 `TASK-GAME-018`，不引入额外玩法范围扩张。

## Validation Plan
- 测试层级：`test_tier_required`
- 验证命令：沿用现有截图闭环与手动验收命令，并回写证据路径。
- 预期结果：微循环视觉增强可被截图 / 视频直接观察到，且 QA 可复核。
- 回归影响范围：game / viewer / playability 体验层。

## Handoff Acknowledgement
- 接收方确认范围：`待 viewer_engineer 回写`
- 接收方确认 ETA：`待 viewer_engineer 回写`
- 接收方新增风险：`待 viewer_engineer 回写`
