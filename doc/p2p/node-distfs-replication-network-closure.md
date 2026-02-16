# Agent World Runtime：Node DistFS 复制网络化收敛（设计文档）

## 目标
- 将 `agent_world_distfs` 中已具备的 `FileReplicationRecord` 能力接入 node 运行时网络路径，形成“可广播、可验签、可恢复”的最小跨节点复制闭环。
- 复用现有 node UDP gossip 主循环，避免引入第二套并行传输栈。
- 以最小代价把 `config.toml` 中的节点密钥用于复制消息签名链路。

## 范围

### In Scope
- **NRX-1：复制消息网络接线**
  - 扩展 node gossip 消息模型，新增 DistFS 复制消息分支。
  - 节点在本地 committed 事件时产出复制记录并广播。
  - 远端节点接收并应用复制记录到本地 CAS/FileStore。
  - 单写者 guard 持久化（重启后可恢复）。

- **NRX-2：复制消息签名/验签**
  - 节点使用 `config.toml` 自举私钥对复制消息签名。
  - 接收端对复制消息执行验签，失败拒绝。
  - `world_viewer_live` 启动路径将自举密钥注入 node runtime。

- **NRX-3：多节点回归与恢复验证**
  - 增加 2 节点复制闭环测试。
  - 增加重启恢复测试（guard 与序列状态恢复）。

### Out of Scope
- 生产级密钥分发与 PKI 信任管理。
- 多写者 CRDT 合并协议。
- 与 libp2p/kad 的完整复制索引协议。

## 接口 / 数据
- `NodeReplicationConfig`：节点复制配置（路径、签名密钥、状态持久化文件）。
- `NodeConfig::with_replication(...)`：启用复制能力。
- gossip 复制消息数据：`FileReplicationRecord + bytes + signature + public_key`。
- 持久化数据：
  - guard 状态（`SingleWriterReplicationGuard`）
  - 本地 writer 序列号状态（单调递增）

## 里程碑
- **NRX-0**：设计文档 + 项目管理文档。
- **NRX-1**：复制消息网络接线 + guard 持久化。
- **NRX-2**：复制消息签名验签接线（消费 config 节点密钥）。
- **NRX-3**：多节点与重启恢复测试收口。

## 风险
- `crates/agent_world_node/src/lib.rs` 文件接近 1200 行，需拆分测试/模块防止超限。
- 当前 node 主循环以最小 UDP gossip 实现，吞吐与可靠性有限；本阶段仅保证功能闭环。
- 验签策略先做最小可用，后续仍需补充节点身份绑定策略。
