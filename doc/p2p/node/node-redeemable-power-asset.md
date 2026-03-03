# Agent World Runtime：可兑现节点资产与电力兑换闭环（设计文档）

## 目标
- 将现有 Node Points 从“统计积分”升级为“可兑现协议资产”，使算力/存储节点可将收益兑换为 Agent 电力。
- 在不破坏现有 PoS + DistFS 能力的前提下，补齐“结算上链、兑换执行、守恒风控、可审计回放”最小经济闭环。
- 用最小可运行方案覆盖前序评估的 8 个关键缺口：
  - 资产层缺失；
  - 结算未上链；
  - 证明链路弱；
  - 质押/罚没生命周期不完整；
  - 奖励链路未接入主运行路径；
  - 缺需求侧支付市场；
  - 身份签名治理待统一；
  - DistFS 仍偏功能闭环、缺生产语义。

## 范围

### In Scope
- **RPA-1：可兑现资产层（PowerCredit）**
  - 新增协议资产 `PowerCredit`（不可直接转账，先支持系统铸造与兑换燃烧）。
  - 资产账本按 `node_id` 维护余额与累计统计（mint/burn/redeem）。

- **RPA-2：Node Points 结算上链**
  - 将 `EpochSettlementReport` 产出的奖励从“内存结果”写入链状态。
  - 增加结算记录结构（epoch -> node mint 明细）并纳入状态快照/回放。

- **RPA-3：兑换动作（PowerCredit -> Agent 电力）**
  - 新增 `RedeemPower` 动作：节点消耗 `PowerCredit`，为指定 Agent 增加电力资源。
  - 兑换公式参数化：`credits_per_power_unit`。

- **RPA-4：守恒与风控**
  - 增加“协议电力储备池”（Protocol Power Reserve）约束兑付，防止无上限增发电力。
  - 增加每 epoch 兑付上限、最小兑换单位、nonce 防重放。

- **RPA-5：运行时接线**
  - 在 `world_viewer_live` / runtime 主循环增加结算与兑换执行接线开关。
  - 保持关闭开关时兼容现有行为。

- **RPA-6：最小需求侧支付入口（MVP）**
  - 增加“系统订单池”概念：按基础存储/在线需求对节点结算分配预算，不引入复杂撮合。
  - 该入口仅用于将“系统补贴池”向“需求驱动收益”过渡。

- **RPA-7：身份与签名治理最小收口**
  - 对结算记录与兑换动作使用统一签名策略（与节点密钥绑定）。
  - 明确 `node_id <-> public_key` 映射检查，拒绝未绑定身份的结算/兑换提交。

- **RPA-8：DistFS 经济语义最小增强**
  - 在现有存储挑战字段基础上增加“可挑战样本来源”标识与失败原因分类。
  - 为后续 PoSt/VRF 接入预留证明字段，不在本阶段实现完整证明协议。

### Out of Scope
- 可自由转账的通用代币经济系统（交易市场、AMM、税率、复杂货币政策）。
- 完整 PoRep/PoSt/VRF 网络协议与多观察点拜占庭证明。
- 完整链上仲裁与法律化争议处理流程。
- 全量生产级存储安全能力（ACL、全链路加密、跨地域复制策略）。

## 接口 / 数据

### 1) 资产与储备账本（草案）
```rust
RewardAssetConfig {
  // NodePoints -> PowerCredit 转换比例
  points_per_credit: u64,
  // PowerCredit -> 电力单位转换比例
  credits_per_power_unit: u64,
  // 每 epoch 最多可兑付电力
  max_redeem_power_per_epoch: i64,
  // 最小兑换电力单位
  min_redeem_power_unit: i64,
}

NodeAssetBalance {
  node_id: String,
  power_credit_balance: u64,
  total_minted_credits: u64,
  total_burned_credits: u64,
}

ProtocolPowerReserve {
  epoch_index: u64,
  available_power_units: i64,
  redeemed_power_units: i64,
}
```

### 2) 结算上链记录（草案）
```rust
NodeRewardMintRecord {
  epoch_index: u64,
  node_id: String,
  source_awarded_points: u64,
  minted_power_credits: u64,
  settlement_hash: String,
  signer_node_id: String,
  signature: String,
}
```

### 3) 兑换动作与事件（草案）
```rust
Action::RedeemPower {
  node_id: String,
  target_agent_id: String,
  redeem_credits: u64,
  nonce: u64,
}

DomainEvent::PowerRedeemed {
  node_id: String,
  target_agent_id: String,
  burned_credits: u64,
  granted_power_units: i64,
  reserve_remaining: i64,
}

DomainEvent::PowerRedeemRejected {
  node_id: String,
  target_agent_id: String,
  redeem_credits: u64,
  reason: String,
}
```

### 4) 关键规则
- `minted_power_credits = floor(awarded_points / points_per_credit)`。
- 兑换前检查：
  - `redeem_credits > 0`；
  - 节点余额充足；
  - `nonce` 严格单调；
  - `granted_power_units >= min_redeem_power_unit`；
  - 目标 epoch 储备池与上限允许。
- 兑换执行：
  - `NodeAssetBalance.power_credit_balance -= redeem_credits`；
  - `ProtocolPowerReserve.available_power_units -= granted_power_units`；
  - `Agent.resource.electricity += granted_power_units`。

### 5) 状态与回放一致性
- 将 `NodeAssetBalance`、`ProtocolPowerReserve`、`NodeRewardMintRecord` 纳入快照。
- 事件重放必须重建相同余额与储备结果；不一致视为状态错误。

## 里程碑
- **RPA-M0**：设计文档与项目管理文档。
- **RPA-M1**：资产账本 + 配置 + 快照持久化。
- **RPA-M2**：Node Points 结算上链铸造接线。
- **RPA-M3**：`RedeemPower` 动作与事件闭环。
- **RPA-M4**：守恒与风控（储备池、额度、nonce）。
- **RPA-M5**：运行时接线（含 `world_viewer_live` 开关）。
- **RPA-M6**：最小需求侧支付入口与回归。
- **RPA-M7**：身份签名治理最小收口。
- **RPA-M8**：DistFS 证明字段增强与文档/devlog 收口。

## 测试策略
- `test_tier_required`：
  - 资产铸造守恒（points -> credits）；
  - 兑换扣账与电力增加一致；
  - 储备池不足拒绝；
  - nonce 重放拒绝；
  - 快照恢复后余额一致。
- `test_tier_full`：
  - 多节点多 epoch 结算与兑换压力场景；
  - 节点重启 + 回放一致性；
  - libp2p 复制下的资产状态一致性抽样。

## 风险
- 参数配置风险：`points_per_credit` 与 `credits_per_power_unit` 配置不当会导致通胀或激励不足。
- 守恒风险：若储备池更新与兑换动作不是原子处理，可能出现短时超发。
- 治理风险：身份绑定策略未完全统一前，需默认严格校验签名与 node_id 映射。
- 演进风险：MVP 需求侧支付入口仍偏系统计划分配，后续需逐步演进到真实订单市场。
