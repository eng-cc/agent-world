# Agent World Runtime：生产级区块链 + P2P FS 路线图（设计文档）

## 目标
- 将当前“可演示的区块链 + P2P FS + 节点收益”实现推进到可生产部署形态。
- 在保持现有可运行能力的同时，优先收敛三类高风险缺口：
  - 共识哈希语义弱（`block_hash` 非交易承诺）；
  - 奖励结算网络消息缺少独立传输签名；
  - 节点主循环与执行主链路仍是桥接模式，缺共识内生执行。
- 输出一条可分阶段验收的工程路线，并在本轮实现 Phase A 的关键基础能力。

## 范围

### In Scope（本轮）
- **PRG-A1：共识区块哈希链式化**
  - 将 `agent_world_node` 的提案 `block_hash` 从字符串占位升级为链式哈希（包含 `parent_hash`）。
  - 持久化引擎最新已提交哈希，确保重启后哈希链连续。
- **PRG-A2：奖励结算 Envelope 传输签名**
  - 为 `RewardSettlementEnvelope` 增加签名字段与验签流程。
  - 在消费侧增加“签名验真 + signer 身份绑定一致性”前置校验。
- **PRG-A3：测试与回归**
  - 补齐新增行为单元测试。
  - 执行 `test_tier_required` 口径回归。

### Roadmap Out of Scope（后续里程碑，不在本轮实现）
- **PRG-B：共识内生执行（替代执行桥接）**
  - 将 `RuntimeWorld` 执行与状态根写入合并到 `NodeRuntime` 主循环。
- **PRG-C：跨节点 DistFS 挑战与证明网络化**
  - 从本地 challenge/self-answer 迁移到跨节点 challenge-response。
- **PRG-D：交易化需求侧支付市场**
  - 从系统预算池演进到订单/结算型支付。
- **PRG-E：运维与治理生产化**
  - 配置治理、密钥轮换、指标告警、灰度发布与回滚手册。

## 接口 / 数据

### 1) 节点区块哈希载荷（新增语义）
```rust
BlockHashPayload {
  version: u8,
  world_id: String,
  height: u64,
  slot: u64,
  epoch: u64,
  proposer_id: String,
  parent_block_hash: String,
}
```
- `block_hash = blake3(cbor(BlockHashPayload))`。
- `parent_block_hash`：已提交高度 `h-1` 的 `block_hash`，创世使用常量 `genesis`。

### 2) 奖励结算网络消息签名（新增字段）
```rust
RewardSettlementEnvelope {
  version: u8,
  world_id: String,
  epoch_index: u64,
  signer_node_id: String,
  signer_public_key_hex: String,
  report: EpochSettlementReport,
  mint_records: Vec<NodeRewardMintRecord>,
  emitted_at_unix_ms: i64,
  signature: String, // rewardsett:v1:<sig_hex>
}
```
- 签名算法：ed25519。
- 验签口径：
  1. envelope 签名有效；
  2. `signer_node_id -> public_key` 与世界状态绑定一致；
  3. 再进入既有 `ApplyNodePointsSettlementSigned` 状态校验。

## 里程碑
- **PRG-M0（本轮）**：路线图文档与任务拆解。
- **PRG-M1（本轮）**：链式 `block_hash` + 持久化兼容。
- **PRG-M2（本轮）**：结算 Envelope 签名/验签 + 消费前置校验。
- **PRG-M3（本轮）**：`test_tier_required` 回归与文档/devlog 收口。
- **PRG-M4（后续）**：共识内生执行。
- **PRG-M5（后续）**：DistFS 证明网络化。
- **PRG-M6（后续）**：支付市场与治理生产化。

## 风险
- 链式哈希切换后，旧测试或历史快照若依赖占位字符串口径，可能出现兼容差异。
- 结算 envelope 双层签名（envelope + mint record）增加排障复杂度，需要清晰日志。
- 后续 PRG-M4/5/6 需要跨 crate 同步演进，若缺少统一验收标准会造成阶段回退。
