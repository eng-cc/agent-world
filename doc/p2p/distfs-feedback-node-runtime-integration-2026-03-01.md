# Agent World Runtime：DistFS Feedback P2P Node Runtime 接入（2026-03-01）设计文档

## 目标
- 在不走共识、无中心化服务器前提下，将 feedback 的“提交 + 传播 + 入库”接入 `agent_world_node` 运行时主循环。
- 让节点具备两种自动行为：
  - 本地提交 feedback 后自动广播 announce。
  - 收到远端 announce 后自动按 hash 拉取 blob 并入本地 feedback store。
- 保持 feedback 现有语义：公开读写、append-only、tombstone 逻辑删除、签名作者控制、审计与限流。

## 范围

### In Scope
- `crates/agent_world_node` 新增 feedback p2p 配置：
  - feedback 本地存储根目录。
  - feedback store 限流/大小限制配置（复用 `FeedbackStoreConfig`）。
  - 每 tick 的 announce drain/publish 上限。
- `NodeRuntime` 内接入 feedback driver：
  - 初始化 `FeedbackStore` + `FeedbackAnnounceBridge`。
  - tick 内执行 `drain incoming announces -> fetch blob -> ingest`。
  - tick 内执行 `flush local announce outbox -> publish`。
- `NodeRuntime` 新增 feedback 提交接口：
  - `submit_feedback` / `append_feedback` / `tombstone_feedback`。
  - 本地 mutation 成功后构造 announce 并入发布队列。
- 单测覆盖：
  - 本地提交触发广播，远端节点自动拉取并入库。
  - 重复 announce 幂等。
  - 未启用 feedback p2p 时提交接口返回明确错误。

### Out of Scope
- feedback 内容审核策略升级。
- 反馈索引查询 API / HTTP 网关。
- 将 feedback 事件绑定到共识最终性。

## 接口 / 数据

### Node 配置（草案）
```rust
NodeFeedbackP2pConfig {
  root_dir: PathBuf,
  store: FeedbackStoreConfig,
  max_incoming_announces_per_tick: usize,
  max_outgoing_announces_per_tick: usize,
}
```

### NodeRuntime 新增行为
- 启动阶段（`start`）：
  - 当 `config.feedback_p2p` 启用且 `replication_network` 存在时，创建 `FeedbackStore` 与 `FeedbackAnnounceBridge`。
- 运行阶段（每 tick）：
  - 入站：`bridge.drain()` -> 按上限逐条处理 -> 通过 replication fetch-blob 协议拉取 -> `ingest_feedback_announce_with_fetcher`。
  - 出站：从本地 outbox 取 announce -> `bridge.publish()`。
- 提交阶段（对外接口）：
  - `submit/append/tombstone` 直接写 `FeedbackStore`。
  - 依据 receipt 调 `build_feedback_announce_from_receipt` 生成 announce，入 outbox。

### 错误处理
- feedback p2p 未启用时，feedback 提交接口返回 `NodeError::Replication`（明确“feedback p2p 未配置”）。
- 单条 announce ingest 失败不阻断 tick；错误计入 runtime `last_error`（遵循现有 runtime 风格）。

## 里程碑
- M1：T0 设计与项目管理文档完成。
- M2：T1 NodeConfig + runtime feedback driver 接入完成。
- M3：T2 feedback 提交 API + 自动广播闭环完成并通过单测。
- M4：T3 回归、文档状态回写与 devlog 收口完成。

## 风险
- fetch-blob 鉴权依赖 replication 配置；若远端开启严格 allowlist，未配置签名请求可能被拒绝。
- gossip announce 可被刷屏；依赖每 tick 上限、feedback store 限流与签名校验兜底。
- 无共识模式下为最终一致，不保证节点间实时一致。
