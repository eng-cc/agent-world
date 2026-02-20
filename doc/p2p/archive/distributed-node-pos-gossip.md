> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node PoS Gossip 协同（设计文档）

## 目标
- 在 `agent_world_node` 的 PoS 主循环基础上，新增跨进程 gossip 协同能力。
- 支持多个节点实例通过 UDP 交换已提交 head 摘要，实现网络视角的 head 追踪。
- 在 `world_viewer_live` 启动参数中暴露 gossip 配置，便于本地多节点联调。

## 范围

### In Scope
- `agent_world_node`：
  - 增加 gossip 配置（绑定地址、peer 列表）。
  - 增加 UDP gossip endpoint（非阻塞接收 + 广播发送）。
  - 在 PoS tick 中广播本地 committed head，并摄取远端 committed head。
  - 在 snapshot 中暴露网络维度状态（network committed height / known peer heads）。
- `world_viewer_live`：
  - 增加 CLI 参数映射 gossip 配置到 `NodeConfig`。
  - 增加参数解析与接线测试。

### Out of Scope
- 完整分布式 attestation 传播与跨节点提案驱动。
- 网络层安全（签名校验、重放防护、加密传输）。
- P2P 发现、NAT 穿透与生产级拓扑管理。

## 接口 / 数据

### NodeGossipConfig
- `bind_addr: SocketAddr`
- `peers: Vec<SocketAddr>`

### Gossip 消息（提交摘要）
- `version`
- `world_id`
- `node_id`
- `height`
- `slot`
- `epoch`
- `block_hash`
- `committed_at_ms`

### Snapshot 增强
- `network_committed_height`
- `known_peer_heads`

## 里程碑
- NPG-1：设计文档与项目管理文档落地。
- NPG-2：`agent_world_node` 实现 gossip endpoint 与状态同步。
- NPG-3：`world_viewer_live` 增加 gossip CLI 接线与测试。
- NPG-4：回归测试、文档状态与 devlog 收口。

## 风险
- UDP 天然不保证可靠投递，网络视角可能短暂滞后。
- 当前 gossip 仅传播 committed 摘要，不包含提案/投票细节。
- 本地多节点同机联调可能遇到端口冲突，需要参数校验与错误提示。
