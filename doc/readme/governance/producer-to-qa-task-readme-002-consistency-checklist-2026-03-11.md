# Role Handoff Brief

审计轮次: 4

## Meta
- Handoff ID: `HANDOFF-README-002-2026-03-11-CHECKLIST`
- Date: `2026-03-11`
- From Role: `producer_system_designer`
- To Role: `qa_engineer`
- Related PRD-ID: `PRD-README-001/002`
- Related Task ID: `TASK-README-002`
- Priority: `P1`

## Goal
- 交付 README 口径巡检清单，让后续 QA / 文档维护能按固定模板复核顶层入口。

## Why Now
- 在没有清单前，README 巡检靠经验执行，很难稳定承接到 `TASK-README-003` 的自动检查。
- 如果不做，后续自动化只能检查断链，仍无法覆盖状态口径与术语漂移。

## Inputs
- 代码 / 文档入口：`doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.md`
- 已完成内容：已冻结 6 个高优巡检项、权威源与失败动作
- 已知约束：本轮仍是人工巡检模板，不含自动脚本
- 依赖前置项：`README.md`、`doc/README.md`、`doc/site/prd.md`、`doc/core/prd.md`

## Expected Output
- 接收方交付物 1：后续链接自动检查任务可复用该清单的检查对象
- 接收方交付物 2：当 README / site 状态口径变化时，可按该模板做人工复核
- 需要回写的文档 / 日志：后续 `TASK-README-003/004` 文档按需引用

## Done Definition
- [x] 满足验收点 1：巡检清单可直接执行
- [x] 满足验收点 2：状态口径检查已被列为高优项
- [x] 补齐测试 / 验证证据

## Risks / Blockers
- 风险：如果后续模块主 PRD 变化过快，人工巡检频率需要提升
- 阻断项：无
- 需要升级给谁：若后续自动检查无法覆盖状态口径，升级给 `producer_system_designer` 与 `qa_engineer` 联合裁定补充项

## Validation
- 建议测试层级：`test_tier_required`
- 建议验证命令：`rg -n "RC-01|RC-02|RC-03|RC-04|RC-05|RC-06|失败动作|权威源" doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.md doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.prd.md`

## Notes
- 接收方确认范围：`已接收 README 巡检清单，可作为后续 QA / 自动检查任务的人工模板`
- 接收方确认 ETA：`same-day`
- 接收方新增风险：`无`
