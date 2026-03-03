# README P1 缺口收口：分布式网络主路径生产化（设计文档）

## 目标
- 收口 P1-1：将 Node 共识消息（proposal/attestation/commit）从“仅 UDP gossip”升级为“libp2p pubsub 主路径”，保留 UDP 兼容兜底。
- 收口 P1-2：将 libp2p request/response 从“单 peer + 无 peer 本地 handler 回退”升级为“多 peer 轮换重试 + 可控本地回退策略”。
- 保持现有 `world_viewer_live` 生产默认拓扑（triad/triad_distributed）可用，并保证 required-tier 回归稳定。

## 范围
- In scope
  - `crates/agent_world_node/src/libp2p_replication_network.rs`
    - request 路由升级为多 peer 轮换 + 失败重试。
    - 增加“无 peer 时本地 handler 回退”开关，默认关闭。
  - `crates/agent_world_node/src/network_bridge.rs`
    - 新增共识 topic endpoint（proposal/attestation/commit）。
  - `crates/agent_world_node/src/lib.rs`
    - 在 node 主循环接入 libp2p 共识消息 ingest/broadcast 主路径。
    - 保留 UDP gossip 兼容路径（network 不可用时兜底）。
  - `crates/agent_world_node/src/pos_engine_gossip.rs`
    - 增加针对 libp2p 共识 endpoint 的本地广播实现。
  - 测试
    - `agent_world_node` 单测覆盖：
      - 多 peer request 重试；
      - 无 peer 默认拒绝与显式本地回退；
      - 共识消息经 libp2p 传播可被远端 ingest。
- Out of scope
  - wasm32 端完整 libp2p 节点协议栈（仍按既有 compile guard）。
  - 共识算法语义重写（PoS 阈值/提议者选择保持不变）。
  - 完整自动运维编排平台。

## 接口 / 数据
### 1) libp2p request 路由
- `Libp2pReplicationNetworkConfig` 新增：
  - `allow_local_handler_fallback_when_no_peers: bool`（默认 `false`）。
- request 语义：
  - 有连接 peer：按轮换顺序选择 peer，失败自动切下一个，直到成功或耗尽。
  - 无连接 peer：默认返回 `NetworkProtocolUnavailable`；仅在开关开启时允许本地 handler 回退。

### 2) 共识 topic（libp2p）
- 主题命名：
  - `aw.<world_id>.consensus.proposal`
  - `aw.<world_id>.consensus.attestation`
  - `aw.<world_id>.consensus.commit`
- 载荷：复用既有 `GossipProposalMessage/GossipAttestationMessage/GossipCommitMessage` JSON 编码。

### 3) Node 主循环广播/消费优先级
- 广播优先级：
  - 有 libp2p 共识 endpoint：走 libp2p。
  - 否则走 UDP gossip。
- 消费优先级：
  - 两路均可 ingest；libp2p 路径用于生产默认主链路，UDP 作为兼容链路。

## 里程碑
- M1：T0 文档冻结（设计 + 项管）。
- M2：T1 libp2p request 路由升级。
- M3：T2 node 共识消息 libp2p 主路径接线。
- M4：T3 测试回归、文档和 devlog 收口。

## 风险
- 网络行为风险：多 peer 重试引入请求状态机复杂度，需严格处理 response/outbound-failure 的 pending 迁移。
- 兼容风险：共识消息双路径（libp2p/UDP）并存阶段可能出现重复消息，需依赖现有高度/哈希幂等守卫。
- 稳定性风险：topic ingest 新增后测试时序更敏感，需避免 flaky 断言。
