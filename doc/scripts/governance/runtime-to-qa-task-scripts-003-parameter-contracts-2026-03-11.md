# Role Handoff Brief

审计轮次: 4

## Meta
- Handoff ID: `HANDOFF-SCRIPTS-003-2026-03-11-PARAM-CONTRACT`
- Date: `2026-03-11`
- From Role: `runtime_engineer`
- To Role: `qa_engineer`
- Related PRD-ID: `PRD-SCRIPTS-002/003`
- Related Task ID: `TASK-SCRIPTS-003`
- Priority: `P1`

## Goal
- 交付高频脚本的最小参数契约与失败语义，让 QA / CI 文档能够引用同一套脚本调用说明。

## Why Now
- 入口层级已冻结，但没有参数契约时，调用者仍要靠 `--help` 或试错使用脚本。
- 如果不做，`skip-*` / `dry-run` 等参数很容易被误读为常规放行路径。

## Inputs
- 代码 / 文档入口：`doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`
- 已完成内容：5 个高频脚本已补最小命令、参数默认值、失败语义与备注
- 已知约束：本轮不覆盖所有低频脚本
- 依赖前置项：脚本 `--help` 输出与当前 shell 行为

## Expected Output
- 接收方交付物 1：testing / QA 文档引用 scripts 时优先复用这些契约表述
- 接收方交付物 2：若发现帮助输出变更，后续据此回写契约文档
- 需要回写的文档 / 日志：后续 testing/manual 或 release gate 文档按需引用

## Done Definition
- [x] 满足验收点 1：高频脚本参数契约表可直接引用
- [x] 满足验收点 2：`dry-run` / `skip-*` 已明确为覆盖变化而非正常放行
- [x] 补齐测试 / 验证证据

## Risks / Blockers
- 风险：`run-viewer-web.sh` 的帮助来自 trunk，后续 trunk 升级会改变帮助文案
- 阻断项：无
- 需要升级给谁：若关键脚本帮助输出与文档频繁漂移，升级给 `runtime_engineer` 做自动校验治理

## Validation
- 建议测试层级：`test_tier_required`
- 建议验证命令：`rg -n "ci-tests.sh|release-gate.sh|build-game-launcher-bundle.sh|run-viewer-web.sh|site-link-check.sh|skip-|dry-run" doc/scripts/governance/script-parameter-contracts-2026-03-11.prd.md doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`

## Notes
- 接收方确认范围：`已接收高频脚本参数契约，可作为 testing / release 文档引用基线`
- 接收方确认 ETA：`same-day`
- 接收方新增风险：`无`
