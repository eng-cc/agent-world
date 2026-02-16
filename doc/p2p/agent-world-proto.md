# Agent World Runtime：`agent_world_proto` 协议类型与 Trait 抽离（设计文档）

## 目标
- 新建独立 crate：`agent_world_proto`，专门承载分布式相关的**协议类型定义**与**抽象 trait**。
- 降低 `agent_world` runtime 对协议定义层的耦合，避免“核心运行时 + 协议契约 + 网络实现”长期混杂。
- 为后续将 `libp2p` 适配器、分布式执行链路进一步拆 crate 提供稳定边界。

## 范围

### In Scope（本次）
- 新建 `crates/agent_world_proto` 并加入 workspace。
- 迁移以下协议类型定义：
  - topic / DHT key 约定、RR 协议名、错误码、请求响应结构。
  - `WorldBlock` / `WorldHeadAnnounce` / `ActionEnvelope` / `SnapshotManifest` 等协议载荷。
  - 共识成员变更相关协议载荷：`ConsensusMembershipChange*`、`ConsensusStatus`、`ConsensusVote`、`HeadConsensusRecord`。
- 迁移以下协议 trait 定义：
  - 网络抽象：`DistributedNetwork`
  - DHT 抽象：`DistributedDht`
- 在 `agent_world` 中保留运行时实现与适配：
  - `InMemoryNetwork` / `InMemoryDht` 继续留在 `agent_world`。
  - `libp2p` 实现继续留在 `agent_world`（本次不搬迁）。
- 通过 wrapper 层保持 `agent_world` 现有 `WorldError` API 语义。

### Out of Scope（本次不做）
- `libp2p_net` 独立成单独 crate。
- observer/bootstrap/validation 等与 `World` 强耦合逻辑拆出 `agent_world`。
- 网络与 DHT 错误模型的语义重构。

## 接口 / 数据

### 新 crate 结构（拟）
- `agent_world_proto::distributed`
  - 协议常量与命名 helper。
  - gossipsub / RR / DHT key 相关类型。
  - 分布式错误码与错误响应结构。
- `agent_world_proto::distributed_net`
  - `NetworkMessage` / `NetworkRequest` / `NetworkResponse` / `NetworkSubscription`。
  - `DistributedNetwork<E>` trait（错误类型泛型）。
- `agent_world_proto::distributed_dht`
  - `ProviderRecord` / `MembershipDirectorySnapshot`。
  - `DistributedDht<E>` trait（错误类型泛型）。
- `agent_world_proto::distributed_consensus`
  - 成员变更协议结构：`ConsensusMembershipChange*`。
  - 共识状态/投票与记录：`ConsensusStatus` / `ConsensusVote` / `HeadConsensusRecord`。

### 兼容策略
- `agent_world` 继续对外导出原有 runtime 命名：
  - `runtime::distributed`、`runtime::distributed_net`、`runtime::distributed_dht`。
- `agent_world` 内部使用 wrapper trait 将错误类型固定为 `WorldError`，尽量减少调用点改动。

## 里程碑
- **P1**：文档与任务拆解完成。
- **P2**：`agent_world_proto` crate 新建并完成协议类型迁移。
- **P3**：trait 迁移 + `agent_world` wrapper 适配 + 编译/测试回归通过。
- **P4**：补齐共识成员变更协议类型迁移并保持 `agent_world` 外部 API 稳定。

## 风险
- trait 泛型化后若 wrapper 不完整，可能导致 trait object 推断歧义。
- 协议类型迁移后若 re-export 漏项，可能引发大量编译失败。
- 现有测试对模块路径较敏感，需注意保持 API 路径稳定。
