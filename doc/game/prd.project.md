# game PRD Project

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-GAME-001 (PRD-GAME-001) [test_tier_required]: 完成 game PRD 改写，建立玩法设计总入口。
- [ ] TASK-GAME-002 (PRD-GAME-001/002) [test_tier_required]: 补齐核心玩法循环（新手/经济/战争）验收矩阵。
- [ ] TASK-GAME-003 (PRD-GAME-002/003) [test_tier_required]: 建立可玩性问题分级与修复闭环模板。
- [ ] TASK-GAME-004 (PRD-GAME-003) [test_tier_required]: 对接发布前可玩性门禁与回归节奏。
- [x] TASK-GAME-005 (PRD-GAME-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。

## 依赖
- doc/game/prd.index.md
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-engineering-architecture.md`
- `doc/playability_test_result/prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-GAME-002
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 game 设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
