# Agent World Runtime：DistFS 反馈 P2P 广播与拉取桥接（2026-03-01）设计文档

## 目标
- 在无中心化服务器场景下，为 feedback 系统补齐“节点间传播”主链路。
- 提供 gossip announce 轻量广播，announce 仅携带元信息与 blob 引用。
- 对端基于 announce 中的 content hash 拉取 blob，验证后入本地 feedback store。
- 保持 feedback 原有签名校验、防重放和 append-only/tombstone 语义。

## 范围

### In Scope
- `crates/agent_world_distfs` 新增 `feedback_p2p` 模块：
  - announce 数据结构与编解码。
  - feedback announce topic 约定。
  - 从本地 mutation receipt 构造 announce（包含 blob_ref）。
  - 对端 ingest：按 hash 拉取 -> hash 校验 -> 解析 root/event -> 回放入库。
- `FeedbackStore` 扩展复制入库接口：
  - `ingest_replicated_root_record`（create）。
  - `ingest_replicated_event_record`（append/tombstone）。
  - 对已有记录支持幂等处理（重复 announce 不应破坏状态）。
- 单测覆盖：
  - create/append/tombstone announce roundtrip。
  - 重放 announce 幂等。
  - blob hash 不匹配拒绝。

### Out of Scope
- DHT provider 策略优化与多源拉取调度。
- 共识层最终性绑定。
- 内容审核、风控策略升级。

## 接口 / 数据

### Topic 命名
- `aw.<world_id>.feedback.announce`

### Announce 结构（草案）
```rust
FeedbackAnnounce {
  version: u8,
  world_id: String,
  feedback_id: String,
  action: FeedbackActionKind, // create|append|tombstone
  event_id: String,
  actor_public_key_hex: String,
  blob_ref: FeedbackBlobRef,
  emitted_at_ms: i64,
}

FeedbackBlobRef {
  path: String,
  content_hash: String,
  size_bytes: u64,
}
```

### Ingest 规则
- 拉取 blob 后先验证 `blake3(blob_bytes) == blob_ref.content_hash`。
- `create`：blob 解析为 `FeedbackRootRecord`，执行复制入库。
- `append/tombstone`：blob 解析为 `FeedbackEventRecord`，执行复制入库。
- 重复 announce：以 `feedback_id + event_id` 去重，幂等返回。

## 里程碑
- M1：T0 文档与任务拆解完成。
- M2：T1 feedback store 复制入库能力完成并通过单测。
- M3：T2 P2P announce/ingest 桥接模块完成并通过单测。
- M4：T3 回归、文档/devlog 收口完成。

## 风险
- 远端 announce 可被垃圾广播；依赖 blob/hash 校验 + store 层签名校验兜底。
- 复制入库与本地限流策略语义不同；复制路径需跳过 IP/pubkey 限流。
- 无共识模式下仅最终一致，不保证全节点实时一致。
