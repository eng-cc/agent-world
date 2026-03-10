# DistFS 公开反馈账本设计

- 对应需求文档: `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.project.md`

## 1. 设计定位
定义基于 `agent_world_distfs` 的公开反馈账本方案：在不引入数据库的前提下支持公开写入/公开读取，并通过 append-only、签名授权、nonce 防重放和审计限流维持基本可信性。

## 2. 设计结构
- 账本存储层：`root.json` + `events/*.json` + `nonces/*` + `audit/*` 构成 append-only 布局。
- 签名授权层：create/append/tombstone 都要求 Ed25519 签名与作者公钥匹配。
- 防滥用层：基于 audit 日志对 IP/pubkey 做时间窗限流、内容大小和附件数量约束。
- 公开读取层：提供 list/get 公共读接口，不做分级读权限。

## 3. 关键接口 / 入口
- `FeedbackRootRecord`
- `FeedbackEvent`
- Ed25519 + nonce 校验
- 公开读 list/get

## 4. 约束与边界
- 不做内容审核、AI 风险过滤与媒体转码。
- 多节点复制一致性优化留给后续专题。
- 本期按需求采用全量公开读，不引入读权限分级。
- append-only 和审计可追溯性优先于复杂查询能力。

## 5. 设计演进计划
- 先落核心存储与签名/nonce 校验。
- 再补限流、审计和公共读接口。
- 最后通过 CLI 闭环与回归收口公开反馈账本。
