# Role Handoff Brief

审计轮次: 5

## Meta
- Handoff ID: `HANDOFF-SITE-009-2026-03-11-CTA-PRIORITY`
- Date: `2026-03-11`
- From Role: `viewer_engineer`
- To Role: `producer_system_designer`
- Related PRD-ID: `PRD-SITE-001/004`
- Related Task ID: `TASK-SITE-009`
- Priority: `P1`

## Goal
- 交付首页与文档入口的 CTA / 信息层级调整结果，确保“技术预览先验证、完整构建后进入”的对外顺序已落地。

## Why Now
- `TASK-SITE-008` 已修正“尚不可玩”口径，但 CTA 仍把路线图/目录放在首位，不足以直接引导访问者先理解当前可验证的预览路径。
- 如果不做，用户仍可能先进入长文档或把构建链路误读为主体验入口。

## Inputs
- 代码 / 文档入口：`site/index.html`、`site/en/index.html`、`site/doc/cn/index.html`、`site/doc/en/index.html`
- 已完成内容：已保持“技术预览（尚不可玩）”真实状态文案不变
- 已知约束：不新增前端框架，不改 `site/assets/app.js` 交互协议
- 依赖前置项：`TASK-SITE-008` 的真实状态口径回写

## Expected Output
- 接收方交付物 1：确认 CTA 排序满足“预览体验优先、构建路径次级”
- 接收方交付物 2：后续若站点继续做转化优化，以该优先级为前提
- 需要回写的文档 / 日志：后续 `core` / `readme` 如需引用对外入口策略，可直接引用本结论

## Done Definition
- [x] 满足验收点 1：CN/EN 首页与文档入口页已同构调整
- [x] 满足验收点 2：真实状态口径未被改弱或稀释
- [x] 补齐测试 / 验证证据

## Risks / Blockers
- 风险：减少“路线图优先”后，部分技术型用户需要多一步才能看到长期规划
- 阻断项：无
- 需要升级给谁：若后续要把 CTA 进一步转向下载/安装，需先由 `producer_system_designer` 重新确认真实可玩状态口径

## Validation
- 建议测试层级：`test_tier_required`
- 建议验证命令：`rg -n "先看技术预览路径|See Preview Verification Path|优先级：预览体验入口优先|Priority: preview experience first" site/index.html site/en/index.html site/doc/cn/index.html site/doc/en/index.html`

## Notes
- 接收方确认范围：`已接收 CTA 优先级调整结果；后续站点转化优化不得绕开“尚不可玩”与“预览先行”口径`
- 接收方确认 ETA：`same-day`
- 接收方新增风险：`无`
