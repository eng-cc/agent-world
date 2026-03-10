# Role Handoff Brief

审计轮次: 4

## Meta
- Handoff ID: `HANDOFF-README-003-2026-03-11-LINK-CHECK`
- Date: `2026-03-11`
- From Role: `producer_system_designer`
- To Role: `qa_engineer`
- Related PRD-ID: `PRD-README-002/003`
- Related Task ID: `TASK-README-003`
- Priority: `P1`

## Goal
- 交付 README 顶层入口链接自动检查脚本，让 QA 能直接验证两份顶层入口文档的本地链接。

## Why Now
- 人工巡检模板已经有了；如果不补自动脚本，顶层断链仍要靠人工发现。
- 如果不做，`TASK-README-004` 的季度节奏也会缺最小自动化抓手。

## Inputs
- 代码 / 文档入口：`scripts/readme-link-check.sh`、`doc/readme/governance/readme-link-check-automation-2026-03-11.project.md`
- 已完成内容：脚本已实现、默认检查 `README.md` 与 `doc/README.md`
- 已知约束：不检查全仓文档和外链
- 依赖前置项：`TASK-README-002` 巡检清单

## Expected Output
- 接收方交付物 1：后续 QA / 文档巡检可直接调用 `scripts/readme-link-check.sh`
- 接收方交付物 2：若 README 顶层断链，按脚本输出回写修复
- 需要回写的文档 / 日志：后续季度审查或 readme 趋势文档

## Done Definition
- [x] 满足验收点 1：脚本可执行并输出 pass/fail
- [x] 满足验收点 2：断链定位字段设计清晰
- [x] 补齐测试 / 验证证据

## Risks / Blockers
- 风险：复杂 Markdown 语法若增多，脚本解析可能需要升级
- 阻断项：无
- 需要升级给谁：若后续要扩展到全仓 Markdown，升级给 `producer_system_designer` 与 `qa_engineer` 联合决定范围

## Validation
- 建议测试层级：`test_tier_required`
- 建议验证命令：`./scripts/readme-link-check.sh`

## Notes
- 接收方确认范围：`已接收 README 顶层入口链接自动检查脚本，可作为后续 readme QA 基线`
- 接收方确认 ETA：`same-day`
- 接收方新增风险：`无`
