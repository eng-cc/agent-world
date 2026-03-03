# Agent World Runtime：奖励结算切换到网络共识主路径原生交易（设计文档）

## 目标
- 将当前 reward runtime 的“旁路直写结算（`apply_node_points_settlement_mint_v2` 直接调用）”切换为世界主执行路径可消费的原生 `Action` 交易。
- 让奖励结算和兑换一样进入 `submit_action -> step -> event -> state apply` 闭环，统一审计与回放语义。
- 保持现有签名治理（`mintsig:v2`、`RewardSignatureGovernancePolicy`）与账本字段兼容，不破坏历史快照读取。

## 范围

### In Scope
- **NSTX-1：新增奖励结算原生交易动作**
  - 新增 `Action::ApplyNodePointsSettlementSigned`，将 epoch 结算报告与签名后的 mint 记录作为交易负载。
  - 该动作走既有 action 主路径，不再通过 runtime 旁路直接改状态。

- **NSTX-2：新增领域事件与状态应用**
  - 新增 `DomainEvent::NodePointsSettlementApplied`，作为奖励结算主路径事件。
  - 在 `WorldState::apply_domain_event` 中实现 mint 账本写入、系统订单池预算扣减、幂等与守恒校验。

- **NSTX-3：交易校验规则**
  - `event_processing` 对结算交易做严格校验：
    - `settlement_hash` 与报告一致；
    - `record.source_awarded_points` 与报告节点奖励一致；
    - `record.minted_power_credits` 不得超过 `awarded / points_per_credit`；
    - 签名通过 `verify_reward_mint_record_signature`；
    - epoch/node 维度无重复，且满足预算约束。
  - 校验失败返回 `DomainEvent::ActionRejected`。

- **NSTX-4：reward runtime 接线切换**
  - `world_viewer_live` reward runtime 从“直接调用 mint API”切换为“构造原生结算交易并提交执行”。
  - 保留 `apply_node_points_settlement_mint*` API 作为兼容接口（用于测试/工具链），但主路径不再依赖。

### Out of Scope
- 跨节点奖励调度器与链上提案器（多节点共识下自动发起奖励结算交易）。
- 将 NodePoints 采样器直接并入 `agent_world_node` 主循环并跨节点同步。
- 奖励市场化定价、可转账代币化和完整清算系统。

## 接口 / 数据

### 新增 Action（草案）
```rust
Action::ApplyNodePointsSettlementSigned {
  report: EpochSettlementReport,
  signer_node_id: String,
  mint_records: Vec<NodeRewardMintRecord>,
}
```

### 新增 DomainEvent（草案）
```rust
DomainEvent::NodePointsSettlementApplied {
  epoch_index: u64,
  signer_node_id: String,
  settlement_hash: String,
  minted_records: Vec<NodeRewardMintRecord>,
}
```

### 状态应用语义（草案）
- 对 `minted_records` 逐条应用：
  - `node_asset_balances[node].power_credit_balance += minted_power_credits`
  - `node_asset_balances[node].total_minted_credits += minted_power_credits`
  - `reward_mint_records.push(record)`
- 预算池（若存在）按记录扣减：
  - `remaining_credit_budget -= minted_power_credits`
  - `node_credit_allocated[node] += minted_power_credits`
- 幂等与一致性：
  - 同 `epoch_index + node_id` 已存在记录时拒绝重复应用。

## 里程碑
- **NSTX-M0**：设计文档 + 项目管理文档。
- **NSTX-M1**：Action/DomainEvent/状态应用落地。
- **NSTX-M2**：reward runtime 接线切换到原生交易。
- **NSTX-M3**：`test_tier_required` 回归、文档状态回写、devlog 收口。

## 风险
- 结算交易负载包含 `EpochSettlementReport` 与 `mint_records`，单笔 payload 体积可能较大；后续需考虑压缩或分片。
- 若预算校验与状态扣减语义不一致，可能出现“校验通过但应用失败”；需以单测锁定规则。
- 从旁路改主路径后，任何字段不兼容都会反映为 action reject，需要明确错误日志与运维排障口径。
