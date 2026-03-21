# oasis7: Moltbook 运营 Runbook（2026-03-21）设计

- 对应需求文档: `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.prd.md`
- 对应项目管理文档: `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.project.md`

审计轮次: 6

## 1. 设计定位
把 Moltbook 从“可发帖的推广渠道”继续收口为“有固定巡检、回复、升级和回写动作的持续运营渠道”，供 `liveops_community` 直接执行。

## 2. 设计结构
- Pre-Publish Gate: 发帖前检查 claim boundary、链接、资产和首评。
- First-24h Loop: 发帖后高频检查 `/home`、通知、帖子评论与互动信号。
- Daily Ops Loop: 常规运营日的固定检查、回复、分桶和记录动作。
- Escalation Matrix: 哪些问题可直接回复，哪些要升级给 `producer_system_designer`、`qa_engineer` 或工程 owner。
- Logging & Review: 当日 `devlog` 回写与周复盘要求。

## 3. 关键接口 / 入口
- `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.prd.md`
- `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.project.md`
- `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.md`
- `doc/readme/governance/readme-moltbook-promotion-plan-2026-03-19.md`
- `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.md`
- `.agents/roles/liveops_community.md`
- `doc/devlog/YYYY-MM-DD.md`

## 4. 约束与边界
- 不在 runbook 中记录 token、本地凭据路径或任何敏感信息。
- 所有回复边界仍以既有技术预览主口径和批准文案为准。
- runbook 只定义执行方法，不替代推广方案和首批文案包。
- 所有升级与回写只使用仓内标准角色名。

## 5. 设计演进计划
- 先固化 Moltbook 日常运营最小闭环。
- 再根据真实互动把高频场景沉淀为更细的模板与指标看板。
- 若未来进入多平台并行运营，再抽象成通用 liveops skeleton。
