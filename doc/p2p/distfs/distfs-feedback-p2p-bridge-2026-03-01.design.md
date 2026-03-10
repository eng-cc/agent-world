# DistFS Feedback P2P Bridge 设计

- 对应需求文档: `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.project.md`

## 1. 设计定位
定义 feedback 账本向 P2P announce/同步桥接的方案：让本地 feedback 变更能够通过轻量 announce 广播到节点网络，并按需 fetch blob 回灌。

## 2. 设计结构
- announce 模型层：从 feedback receipt 构建 announce/outbox。
- bridge 层：负责 publish/drain incoming announce。
- fetch 回灌层：入站 announce 经 replication fetch-blob 拉取并 ingest。
- 限流保护层：每 tick 入站/出站 announce 数量受控，失败不阻断主循环。

## 3. 关键接口 / 入口
- feedback announce/outbox
- `FeedbackAnnounceBridge`
- replication fetch-blob
- ingest feedback announce

## 4. 约束与边界
- 不把 feedback 事件绑定到共识最终性。
- 桥接重点是 announce + fetch 闭环，不扩展查询网关。
- 单条失败应局部降级，不能拖垮 tick。
- 安全校验依赖现有 replication 签名/allowlist 体系。

## 5. 设计演进计划
- 先定义 announce 与 bridge。
- 再接 fetch + ingest 闭环。
- 最后用节点 runtime 与测试验证桥接稳定性。
