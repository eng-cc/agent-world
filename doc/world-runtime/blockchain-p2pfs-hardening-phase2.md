# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 2）设计文档

## 目标
- 在 Phase 1 网络闭环基础上，补齐“最小可信 + 重启可恢复”能力。
- 将节点密钥真正接入 PoS gossip 主链路（proposal/attestation/commit 签名验签）。
- 为 Node PoS 主循环增加状态持久化，支持重启后继续推进高度而非从 1 开始。

## 范围

### In Scope
- **HP2-1：Node PoS gossip 签名/验签闭环**
  - proposal/attestation/commit 消息增加签名字段（公钥 + 签名）。
  - 节点发送侧使用本地签名密钥签名。
  - 接收侧在开启签名策略时强制验签，不通过则拒绝消息。
  - 保持旧消息兼容（未开启签名策略时可接收无签名消息）。

- **HP2-2：Node PoS 状态持久化**
  - 新增本地状态快照文件（基于节点 replication root）。
  - 每 tick 后持久化主循环关键进度（height/slot/广播游标）。
  - 节点启动时尝试恢复持久化状态。

- **HP2-3：回归与收口**
  - 补齐签名验签测试与重启恢复测试。
  - 更新项目文档状态与 devlog。

### Out of Scope
- ActionEnvelope/WorldHeadAnnounce 在 `agent_world_consensus` 的签名算法替换（HMAC -> ed25519）与跨 crate 统一签名接口重构。
- observer 运行态指标到 viewer 运维面板的 UI 展示接线（进入下一阶段）。
- 多节点身份治理（公钥绑定、信任根、吊销）完整治理闭环。

## 接口 / 数据

### Gossip 消息签名字段（草案）
- `public_key_hex: Option<String>`
- `signature_hex: Option<String>`

### PoS 持久化文件（草案）
- 路径：`<replication_root>/node_pos_state.json`
- 字段：
  - `next_height`
  - `next_slot`
  - `committed_height`
  - `network_committed_height`
  - `last_broadcast_proposal_height`
  - `last_broadcast_local_attestation_height`
  - `last_broadcast_committed_height`

## 里程碑
- **HP2-0**：设计文档 + 项目管理文档。
- **HP2-1**：Node PoS gossip 签名/验签闭环完成。
- **HP2-2**：Node PoS 状态持久化与恢复完成。
- **HP2-3**：回归测试、文档与 devlog 收口。

## 风险
- 签名严格校验开启后，混合版本节点（部分未签名）会被拒绝，需要渐进开关策略。
- 持久化频率与磁盘写放大会影响高频 tick；本阶段先保证正确性。
- 状态文件损坏时需容错回退到默认启动路径，避免阻塞节点启动。
