# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 1）设计文档

## 目标
- 以最小阶段把“基础可用”提升为“可跨进程联调、可网络接线、可扩展”的链路。
- 聚焦两个主缺口：
  - `world_viewer_live` 上层未接入 `libp2p` 复制网络。
  - Node PoS 仍缺 proposal/attestation 的 gossip 传播闭环。
- 在不破坏现有 UDP commit/DistFS 复制闭环的前提下，提供可灰度启用的新路径。

## 范围

### In Scope
- **HP1-1：`world_viewer_live` 注入 libp2p replication 网络**
  - 新增 CLI 参数用于配置 replication libp2p listen/bootstrap/topic。
  - 在启动 node runtime 时创建 `Libp2pNetwork`，并通过 `NodeReplicationNetworkHandle` 注入。
  - 保持现有 UDP gossip 路径兼容（可并存，复制优先走注入网络）。

- **HP1-2：Node PoS proposal/attestation gossip 扩展**
  - 扩展 UDP gossip 消息类型，增加 proposal/attestation。
  - 提案节点广播 proposal；本地节点广播自身 attestation。
  - 接收端能够消费远端 attestation 并用于推进 pending proposal。

- **HP1-3：测试与文档收口**
  - 补齐 CLI 解析与启动接线测试。
  - 补齐 node gossip proposal/attestation 的单测或集成回归。
  - 回写项目文档状态与 devlog。

### Out of Scope
- Action/Head 的 ed25519 全链路签名替换（本阶段不改共识签名模型）。
- Node PoS 完整状态持久化与重启续跑（另起阶段）。
- 复制与共识统一到同一 libp2p topic 拓扑（本阶段只扩复制上层接线 + 共识 gossip 扩展）。

## 接口 / 数据

### `world_viewer_live` 新增参数（草案）
- `--node-repl-libp2p-listen <multiaddr>`（可重复）
- `--node-repl-libp2p-peer <multiaddr>`（可重复）
- `--node-repl-topic <topic>`（可选）

### Node Gossip 消息扩展（草案）
- `GossipProposalMessage`
  - `version/world_id/node_id/height/slot/epoch/block_hash/proposed_at_ms`
- `GossipAttestationMessage`
  - `version/world_id/node_id/height/slot/epoch/block_hash/validator_id/approve/source_epoch/target_epoch/voted_at_ms/reason`

### 行为约束
- 仅 `world_id` 匹配的 proposal/attestation 才可被消费。
- attestation 仅作用于当前 pending 且 `height + block_hash` 匹配的提案。
- proposal/attestation 传播不改变现有 committed head 广播语义。

## 里程碑
- **HP1-0**：设计文档 + 项目管理文档。
- **HP1-1**：`world_viewer_live` libp2p replication 注入完成。
- **HP1-2**：Node proposal/attestation gossip 扩展完成。
- **HP1-3**：测试回归、文档与 devlog 收口。

## 风险
- `crates/agent_world_node/src/lib.rs` 已接近 1200 行，需要拆分模块防止超限。
- libp2p 参数配置错误会导致“网络未连通但进程可启动”，需提供明确错误与测试覆盖。
- proposal/attestation 广播引入后，消息乱序可能造成短时 pending；需保持幂等处理。
