# DistFS Feedback 接入 Node Runtime 设计

- 对应需求文档: `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.project.md`

## 1. 设计定位
定义 feedback p2p 在 Node Runtime 中的接入方案：让节点能够在 tick 内 drain/publish feedback announce，并复用 replication store 完成分布式反馈同步。

## 2. 设计结构
- 配置扩展层：`NodeFeedbackP2pConfig` 提供 store 限流和每 tick announce 上限。
- runtime driver 层：NodeRuntime 启动时创建 `FeedbackStore` 与 `FeedbackAnnounceBridge`。
- 入站/出站层：每 tick drain incoming announces + fetch blob ingest，同时发布本地 outbox。
- 提交闭环层：`submit/append/tombstone` 写 store 后自动生成 announce 并入 outbox。

## 3. 关键接口 / 入口
- `NodeFeedbackP2pConfig`
- `FeedbackStore`
- `FeedbackAnnounceBridge`
- `build_feedback_announce_from_receipt`

## 4. 约束与边界
- feedback 内容审核、查询 API 和共识最终性绑定不在本轮范围。
- 单条 announce ingest 失败不阻断 tick，只记 runtime error。
- fetch-blob 鉴权依赖 replication 配置，需复用现有安全链路。
- 无共识模式下只保证最终一致，不承诺实时一致。

## 5. 设计演进计划
- 先补 NodeConfig 和 runtime feedback driver。
- 再接 feedback 提交接口与自动广播。
- 最后通过测试和拆分 `node/lib.rs` 收口阶段实现。
