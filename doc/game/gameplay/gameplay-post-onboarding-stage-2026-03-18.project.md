# PostOnboarding 阶段目标链（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md`

审计轮次: 1

## 任务拆解

- [x] TASK-GAMEPLAY-POD-001 (`PRD-GAME-007`) [test_tier_required]: 冻结 `PostOnboarding` 阶段 PRD / design / project，完成 `game` 根入口、索引、顶层设计主文档与 devlog 挂载。
- [ ] TASK-GAMEPLAY-POD-002 (`PRD-GAME-007`) [test_tier_required]: `viewer_engineer` / `runtime_engineer` 对齐阶段机与目标选择信号契约，明确 `FirstSessionLoop` 完成、主目标来源、阻塞分类与恢复状态。
- [ ] TASK-GAMEPLAY-POD-003 (`PRD-GAME-007`) [test_tier_required]: `viewer_engineer` 落地阶段切换卡、主目标卡、阶段完成卡与恢复逻辑，关闭当前 `#46` 的 UI / 产品缺口。
- [ ] TASK-GAMEPLAY-POD-004 (`PRD-GAME-007`) [test_tier_required]: `qa_engineer` 新增 `#46` required-tier 验证、Web 闭环脚本 / 证据和 playability 卡片，产出通过 / 阻断结论。

## 依赖

- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.md`
- `doc/playability_test_result/topics/industrial-onboarding-required-tier-cards-2026-03-15.md`
- `testing-manual.md`

## 状态

- 更新日期: 2026-03-18
- 当前状态: planned
- 当前 owner: `producer_system_designer`
- 下一任务: `TASK-GAMEPLAY-POD-002`
- 阻断条件:
  - 若阶段目标缺少进度 / 阻塞 / 下一步三要素，则不得认定 `#46` 已关闭。
  - 若只有 UI 文案，没有同会话阶段切换与 required-tier 证据，则不得给出 go 结论。
- 说明:
  - 本 project 文档只承接 `PostOnboarding` 阶段目标链，不重写工业 / 治理 / 战争底层规则。
  - v1 先以工业优先的首个持续能力里程碑关闭 `#46`，再扩展中循环分支表达。
