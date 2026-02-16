# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈（设计文档）

## 目标
- 将 `crates/agent_world_node` 的 DistFS 复制消息从“仅 UDP gossip”迁移为“优先使用 `distributed_net` 统一网络抽象（可接入 libp2p 实现）”。
- 保持现有 UDP 共识提交广播不变，先迁移复制数据通道，降低改造风险。
- 提供可由上层集成的复制 topic 与网络注入能力，支持多节点复制走统一网络栈。

## 范围

### In Scope
- **NRM-1：Node 复制网络桥接**
  - 在 `NodeRuntime` 增加可注入 `DistributedNetwork` 的复制通道。
  - 复制消息发布/订阅走统一 topic；未配置网络时保持 UDP 复制回退。
  - 增加基于 `InMemoryNetwork` 的复制回归测试。

- **NRM-2：Node 侧外部注入接线增强**
  - 增加 replication topic 配置能力，支持按 world 隔离网络通道。
  - 保持 `NodeRuntime` 通过 trait 注入网络对象，供上层（含 libp2p 适配层）接入。
  - 与现有节点密钥签名链路兼容。

- **NRM-3：收口回归**
  - 覆盖 node 测试回归与编译检查。
  - 更新项目文档状态与 devlog。
- **NRM-4：crate 目录路径去歧义**
  - 将 `crates/node` 重命名为 `crates/agent_world_node`。
  - 同步 workspace 与依赖路径配置。

### Out of Scope
- 将 PoS commit gossip 也迁移到 libp2p（本轮仅复制消息迁移）。
- DHT/provider 索引协议重构。
- 生产级 NAT 穿透与复杂拓扑自动发现策略。

## 接口 / 数据
- `NodeRuntime::with_replication_network(...)`：注入统一网络对象。
- `NodeReplicationNetworkHandle::with_topic(...)`：配置复制 topic。
- 复制 topic：`aw.<world_id>.replication`（默认）。
- 复制消息仍沿用 `GossipReplicationMessage` 结构，序列化为 JSON payload 发布到 topic。

## 里程碑
- **NRM-0**：设计文档 + 项目管理文档。
- **NRM-1**：Node 支持统一网络复制通道 + InMemory 回归。
- **NRM-2**：Node 外部注入接线增强（topic 配置）。
- **NRM-3**：回归收口与文档状态完成。
- **NRM-4**：crate 路径标准化（`crates/agent_world_node`）。

## 风险
- `crates/agent_world_node/src/lib.rs` 行数压力较高，改动需继续控制在 1200 行以下。
- 双通道（UDP+网络）并存期间需避免重复应用同一复制记录，依赖单调序列守卫兜底。
- 工作区存在 `agent_world_net -> agent_world -> agent_world_node` 依赖链约束，上层 libp2p 适配需避免形成反向循环依赖。
