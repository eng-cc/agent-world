# Agent World Runtime：分布式成员目录 DHT 快照与恢复（设计文档）

## 目标
- 将成员目录（validators + quorum）写入 DHT，减少节点离线后仅依赖 gossip 才能追平目录的问题。
- 在节点重启或冷启动阶段，从 DHT 读取最近成员目录并恢复到本地 `QuorumConsensus`。
- 保持与现有成员广播同步链路兼容：广播用于实时收敛，DHT 用于恢复兜底。

## 范围

### In Scope（本次实现）
- 扩展 `DistributedDht`：支持成员目录快照 `put/get`。
- 为协议命名补充成员目录 DHT key：`/aw/world/<world_id>/membership`。
- 在 `MembershipSyncClient` 新增能力：
  - `publish_membership_change_with_dht`：广播后同步写入 DHT 快照。
  - `restore_membership_from_dht`：从 DHT 读取并应用 `ReplaceValidators` 恢复。
- 覆盖单测：DHT 快照存取、发布落盘、缺省恢复、恢复应用。

### Out of Scope（本次不做）
- 成员目录多版本历史与回滚。
- 成员目录签名验真与反重放窗口。
- 周期性 DHT 快照压缩或多副本冗余策略。

## 接口 / 数据

### DHT Key
- `dht_membership_key(world_id) -> /aw/world/<world_id>/membership`

### DHT 快照结构
- `MembershipDirectorySnapshot`
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `reason`
  - `validators`
  - `quorum_threshold`

### DHT 抽象
- `DistributedDht::put_membership_directory(world_id, snapshot)`
- `DistributedDht::get_membership_directory(world_id)`

### 成员同步客户端
- `MembershipSyncClient::publish_membership_change_with_dht(...)`
- `MembershipSyncClient::restore_membership_from_dht(...)`

### 恢复规则
- DHT 存在快照：转换为 `ReplaceValidators` 请求并调用 `QuorumConsensus::apply_membership_change`。
- DHT 无快照：返回 `None`，保持本地成员目录不变。
- 若本地存在 pending 共识记录，沿用既有保护规则拒绝目录切换。

## 里程碑
- **MD1**：扩展 DHT 成员目录快照接口与 key helper。
- **MD2**：实现 membership 广播 + DHT 快照联动写入。
- **MD3**：实现从 DHT 恢复成员目录能力。
- **MD4**：补齐单元测试并更新项目文档/开发日志。

## 风险
- 当前恢复逻辑默认信任 DHT 最近记录，恶意写入防护依赖后续签名机制。
- 仅保留最新快照，不含历史演进链路，排障时需要结合 devlog/治理日志。
- 离线恢复和实时广播在极端网络抖动下可能短暂不一致，需依赖幂等覆盖收敛。
