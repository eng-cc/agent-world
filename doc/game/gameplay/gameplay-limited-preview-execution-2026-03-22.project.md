# Gameplay 受控 Limited Preview 执行（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-limited-preview-execution-2026-03-22.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-limited-preview-execution-2026-03-22.prd.md`

审计轮次: 1

## 任务拆解

- [x] TASK-GAMEPLAY-LTP-001 (`PRD-GAME-010`) [test_tier_required]: `producer_system_designer` 已建立受控 limited preview 执行专题，冻结本轮阶段前提、外放边界、handoff 与回流要求，并挂载到 game 根入口与索引。
- [ ] TASK-GAMEPLAY-LTP-002 (`PRD-GAME-010`) [test_tier_required]: `liveops_community` 执行 1 轮 invite-only builder callout，按 `T+15m / T+1h / T+4h / T+24h` 巡检并将信号按 `Blocking / Opportunity / Idea` 回流。
- [ ] TASK-GAMEPLAY-LTP-003 (`PRD-GAME-010`) [test_tier_required]: `qa_engineer` 基于 unified gate 当前真值、本轮新增信号与最近 7 天趋势，输出 `QA Weekly / Event Verdict` 并判断是否继续维持 `pass`。
- [ ] TASK-GAMEPLAY-LTP-004 (`PRD-GAME-010`) [test_tier_required]: `producer_system_designer` 读取 liveops 摘要与 QA 守门结论，正式决定 `continue / hold / reassess`，并回写 game 根 project 与 devlog。

## 依赖

- `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.prd.md`
- `doc/testing/evidence/closed-beta-candidate-release-gate-2026-03-22.md`
- `doc/readme/governance/readme-closed-beta-candidate-runbook-2026-03-22.prd.md`
- `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.md`
- `doc/playability_test_result/templates/closed-beta-candidate-feedback-log-guide-2026-03-22.md`
- `doc/playability_test_result/templates/closed-beta-candidate-incident-templates-2026-03-22.md`
- `testing-manual.md`

## 状态

- 更新日期: 2026-03-22
- 当前状态: in_progress
- 当前 owner: `producer_system_designer`
- 下一任务: `TASK-GAMEPLAY-LTP-002`
- 已完成补充:
  - `TASK-GAMEPLAY-LTP-001` 已新增 `doc/game/gameplay/gameplay-limited-preview-execution-2026-03-22.{prd,design,project}.md`，并向 `liveops_community` / `qa_engineer` 发出标准 handoff。
- 阻断条件:
  - 若 invite-only 外放产生未被及时纠偏的高可见度 claim drift，则不得继续扩大本轮 limited preview 节奏。
  - 若 unified gate 任一硬 lane 回退为 `block`，则 `TASK-GAMEPLAY-LTP-003` 必须建议暂停本轮执行。
  - 若首批信号无法归档到反馈/事故模板，则 producer 不得给出“limited preview 已稳定”的结论。
- 说明:
  - 本专题关注“真实执行闭环”，不是“阶段升级”。
  - `TASK-GAMEPLAY-LTP-002` 只允许 invite-only builder callout，不允许公开发布式扩散。
  - `TASK-GAMEPLAY-LTP-003` 的目标不是放大承诺，而是持续证明当前预览链路没有退化。
