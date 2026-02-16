# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈（设计文档）

## 目标
- 将 `crates/node` 的 DistFS 复制消息从“仅 UDP gossip”迁移为“优先使用 `agent_world_net::DistributedNetwork`（可由 `Libp2pNetwork` 提供）”。
- 保持现有 UDP 共识提交广播不变，先迁移复制数据通道，降低改造风险。
- 在 `world_viewer_live` 启动路径提供最小 libp2p 配置入口，支持多节点复制通过统一网络栈跑通。

## 范围

### In Scope
- **NRM-1：Node 复制网络桥接**
  - 在 `NodeRuntime` 增加可注入 `DistributedNetwork` 的复制通道。
  - 复制消息发布/订阅走统一 topic；未配置网络时保持 UDP 复制回退。
  - 增加基于 `InMemoryNetwork` 的复制回归测试。

- **NRM-2：world_viewer_live 接线 libp2p**
  - 新增 CLI 参数用于配置 replication libp2p listen/bootstrap 地址。
  - 启动时构建 `Libp2pNetwork` 并注入 `NodeRuntime`。
  - 与现有节点密钥签名链路兼容。

- **NRM-3：收口回归**
  - 覆盖 node + world_viewer_live 测试回归与编译检查。
  - 更新项目文档状态与 devlog。

### Out of Scope
- 将 PoS commit gossip 也迁移到 libp2p（本轮仅复制消息迁移）。
- DHT/provider 索引协议重构。
- 生产级 NAT 穿透与复杂拓扑自动发现策略。

## 接口 / 数据
- `NodeRuntime::with_replication_network(...)`：注入统一网络对象。
- 复制 topic：`aw.<world_id>.replication`（默认）。
- 复制消息仍沿用 `GossipReplicationMessage` 结构，序列化为 JSON payload 发布到 topic。

## 里程碑
- **NRM-0**：设计文档 + 项目管理文档。
- **NRM-1**：Node 支持统一网络复制通道 + InMemory 回归。
- **NRM-2**：world_viewer_live libp2p 启动接线。
- **NRM-3**：回归收口与文档状态完成。

## 风险
- `crates/node/src/lib.rs` 行数压力较高，改动需继续控制在 1200 行以下。
- libp2p 初始化失败会影响节点启动，需要明确错误信息并保持回退路径可控。
- 双通道（UDP+网络）并存期间需避免重复应用同一复制记录，依赖单调序列守卫兜底。
