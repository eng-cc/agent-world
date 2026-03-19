# Agent World: Moltbook 首批发帖草案包（2026-03-19）设计

- 对应需求文档: `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.prd.md`
- 对应项目管理文档: `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.project.md`

审计轮次: 6

## 1. 设计定位
把 Moltbook 平台方案继续推进成一组可直接排期、可审核、可在评论区复用的短帖文案包。

## 2. 设计结构
- Meta: owner、review status、source plan。
- Post Queue: 6 条主贴，按冷启动顺序排序。
- Comment Adds: 每条主贴配 1 条首评补充。
- Reply Templates: 覆盖状态、试玩、集成、合作等高频提问。
- Guardrails: 禁宣称项、发前复核表。

## 3. 关键接口 / 入口
- `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.prd.md`
- `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.project.md`
- `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.md`
- `doc/readme/governance/readme-moltbook-promotion-plan-2026-03-19.md`

## 4. 约束与边界
- 文案必须短、原生、可评论延展。
- 不把工程术语堆成长说明书。
- 必须始终保留 `technical preview / not playable yet` 边界。
- 每条文案只推动一个 CTA，避免贪多。

## 5. 设计演进计划
- 先沉淀冷启动 6 条。
- 再按真实互动反馈派生更强变体。
- 如未来扩展到别的平台，再抽象成跨平台短帖模板。
