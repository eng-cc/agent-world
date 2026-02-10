# Agent World Runtime：分布式成员目录同步与变更广播（设计文档）

## 目标
- 在多节点部署中同步共识验证者目录，避免各节点成员视图分叉。
- 将本地成员变更结果广播到网络，让其他节点可按同一目录收敛。
- 提供可测试的订阅、发布、同步闭环接口，便于在 InMemory/libp2p 网络下复用。

## 范围

### In Scope（本次实现）
- 新增成员目录广播 topic：`aw.<world_id>.membership`。
- 定义成员目录广播消息结构 `MembershipDirectoryAnnounce`。
- 提供 `MembershipSyncClient`：
  - 发布成员变更结果广播。
  - 订阅并 drain 广播消息。
  - 将广播目录同步到本地 `QuorumConsensus`。
- 增加单元测试覆盖：发布/订阅、同步应用、幂等重复消息处理。

### Out of Scope（本次不做）
- 成员目录的 DHT 持久化与历史版本追踪。
- 成员变更的签名验真与反重放窗口。
- 跨世界批量同步与压缩广播。

## 接口 / 数据

### Topic
- `topic_membership(world_id) -> aw.<world_id>.membership`

### 广播消息
- `MembershipDirectoryAnnounce`
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `reason`
  - `validators`
  - `quorum_threshold`

### 同步客户端
- `MembershipSyncClient::publish_membership_change(world_id, request, result)`
- `MembershipSyncClient::subscribe(world_id)`
- `MembershipSyncClient::drain_announcements(subscription)`
- `MembershipSyncClient::sync_membership_directory(subscription, consensus)`

### 同步规则
- 同步时将广播目录转换为 `ReplaceValidators` 请求并调用 `QuorumConsensus::apply_membership_change`。
- 若目录已一致，返回 `ignored`（幂等）；若目录变更成功，计入 `applied`。
- 若本地存在 pending 提案，沿用共识层保护策略，阻断目录切换。

## 里程碑
- **CS1**：定义成员目录 topic 与广播数据结构。
- **CS2**：实现发布/订阅与 drain。
- **CS3**：实现目录同步应用与幂等处理。
- **CS4**：单元测试与项目文档/日志更新。

## 风险
- 当前广播消息未做签名验真，仍需依赖上层可信网络与租约约束。
- 若节点长期离线，恢复后需要额外补偿同步（后续可结合 DHT 快照）。
- pending 提案期间拒绝目录切换会降低灵活性，但可避免中间态不一致。
