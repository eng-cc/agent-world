# game PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-GAME-001 (PRD-GAME-001) [test_tier_required]: 完成 game PRD 改写，建立玩法设计总入口。
- [x] TASK-GAME-002 (PRD-GAME-001/002) [test_tier_required]: 补齐核心玩法循环（新手/经济/战争）验收矩阵。
- [x] TASK-GAME-003 (PRD-GAME-002/003) [test_tier_required]: 建立可玩性问题分级与修复闭环模板。
- [ ] TASK-GAME-004 (PRD-GAME-003) [test_tier_required]: 对接发布前可玩性门禁与回归节奏。
- [x] TASK-GAME-005 (PRD-GAME-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-GAME-006 (PRD-GAME-004) [test_tier_required]: 新增微循环反馈可见性 PRD 与项目文档，完成文档树挂载。
- [x] TASK-GAME-007 (PRD-GAME-004) [test_tier_required]: 落地 runtime 协议与 viewer 反馈闭环并完成回归验证（子任务 `TASK-GAMEPLAY-MLF-001/002/003/004` 已全部完成，见 `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.project.md`）。
- [x] TASK-GAME-008 (PRD-GAME-005) [test_tier_required]: 新增“分布式执行共识/治理共识/身份与反女巫”专题 PRD 与项目管理文档，完成根文档追踪映射。
- [x] TASK-GAME-009 (PRD-GAME-005) [test_tier_required]: 落地 tick 证书链与 `state_root/events_hash` 一致性校验实现（含 replay/save-load 闭环）。
- [x] TASK-GAME-010 (PRD-GAME-005) [test_tier_required]: 落地治理 `timelock + epoch` 生效门禁与紧急刹车/否决约束。
- [x] TASK-GAME-011 (PRD-GAME-005) [test_tier_required + test_tier_full]: 落地身份信誉/抵押权重、女巫检测与惩罚申诉闭环。

## 依赖
- doc/game/prd.index.md
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-engineering-architecture.md`
- `doc/playability_test_result/prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-06
- 当前状态: active
- 下一任务: TASK-GAME-004（对接发布前可玩性门禁与回归节奏）
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- ROUND-002 进展: gameplay 子簇主从化完成，`TASK-GAMEPLAY-MLF-001/002/003/004` 与 `TASK-GAME-007` 已闭环；分布式长期在线专题已完成设计建档（`TASK-GAME-008`）与执行共识首个实现切片（`TASK-GAME-009`）。
- ROUND-003 进展: `TASK-GAME-010` 已完成，治理 `Queued + timelock/epoch` 门禁与紧急控制（刹车/否决）状态机已落地并通过定向回归。
- ROUND-004 进展: `TASK-GAME-011` 已完成，`TASK-GAME-DCG-007/008`（身份权重快照 + 女巫惩罚申诉闭环）已落地并通过治理/协议/持久化/审计回归。
- ROUND-005 进展: `TASK-GAME-DCG-009` 已完成，P2P 长稳脚本新增共识哈希一致性门禁并通过 triad+chaos 烟测。
- ROUND-006 进展: `TASK-GAME-DCG-010` 已完成，发布门禁报告与回滚预案已输出（含 `soak_release` 基线证据）。
- ROUND-007 进展: `TASK-GAME-002` 已完成，根 PRD 新增“新手/经济/战争”三循环验收矩阵（含 Given/When/Then、规则边界、证据事件、`test_tier_required` 入口与失败处置）。
- ROUND-008 进展: `TASK-GAME-003` 已完成，根 PRD 新增 `P0~P3` 分级标准、`opened -> triaged -> fixing -> verified -> closed/deferred` 闭环模板与强制约束（含豁免门禁）。
- 说明: 本文档仅维护 game 设计执行状态；过程记录在 `doc/devlog/2026-03-05.md` 与 `doc/devlog/2026-03-06.md`。
