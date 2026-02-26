# Agent World 主链 Token 分配与发行机制（草案）

## 目标
- 补齐主链级 tokenomics 缺口，明确“创世分配 + 持续发行 + 销毁回收 + 治理约束”的完整闭环。
- 将现有 `NodePoints/PowerCredit` 结算链路与主链原生 Token 的经济层解耦并可桥接，避免奖励体系重复记账。
- 提供可审计、可回放、可治理的参数化机制，保证升级不破坏历史快照与共识重放。

## 范围

### In Scope
- 主链原生 Token（暂定 `AWT`）的创世分配规则与分配桶定义。
- 线性/悬崖（cliff）解锁规则、可领取接口与防重放约束。
- epoch 级增发规则（通胀区间 + 质押率反馈）与分配比例。
- 销毁与回收规则（Gas 基础费销毁、罚没销毁、协议金库留存比例）。
- 参数治理边界（可调范围、提案生效延迟、回放一致性）。
- 审计报表与核心不变量。

### Out of Scope
- 跨链桥、二级市场 AMM、做市策略、CEX/DEX 上线流程。
- 法币入口、KYC/合规与法律主体配置。
- 复杂税收策略（动态税率、分层税阶、地区税制差异）。
- NodePoints/PowerCredit 到 AWT 的自动兑换市场（本草案仅定义桥接接口占位）。

## 接口 / 数据

### 1) 主配置（草案）
```rust
MainTokenConfig {
  symbol: String,                  // "AWT"
  decimals: u8,                    // 9
  initial_supply: u128,            // 创世总量
  max_supply: Option<u128>,        // None 表示无硬顶
  inflation_policy: InflationPolicy,
  issuance_split: IssuanceSplitPolicy,
  burn_policy: BurnPolicy,
}

InflationPolicy {
  base_rate_bps: u32,              // 默认 400 = 4.00%
  min_rate_bps: u32,               // 默认 200 = 2.00%
  max_rate_bps: u32,               // 默认 800 = 8.00%
  target_stake_ratio_bps: u32,     // 默认 6000 = 60%
  stake_feedback_gain_bps: u32,    // 默认 1000
  epochs_per_year: u32,            // 默认 365
}

IssuanceSplitPolicy {
  staking_reward_bps: u32,         // 默认 6000
  node_service_reward_bps: u32,    // 默认 2000
  ecosystem_pool_bps: u32,         // 默认 1500
  security_reserve_bps: u32,       // 默认 500
}

BurnPolicy {
  gas_base_fee_burn_bps: u32,      // 默认 3000
  slash_burn_bps: u32,             // 默认 5000
  module_fee_burn_bps: u32,        // 默认 2000
}
```

### 2) 创世分配桶（草案）
```rust
GenesisAllocationBucket {
  bucket_id: String,
  ratio_bps: u32,                  // 分配比例，所有 bucket 之和必须 10000
  vesting: VestingPolicy,
  recipient: AllocationRecipient,  // address / treasury / contract
}

VestingPolicy {
  cliff_epochs: u64,
  linear_unlock_epochs: u64,
  start_epoch: u64,
}
```

建议默认桶：
- `consensus_bootstrap_pool`: 35%
- `ecosystem_growth_pool`: 25%
- `protocol_treasury`: 15%
- `core_contributor_vesting`: 15%
- `community_bootstrap`: 10%

### 3) 发行公式（草案）
- 质押反馈通胀率：
  - `effective_rate = clamp(base + gain * (target_stake_ratio - actual_stake_ratio), min, max)`
- 单 epoch 增发：
  - `epoch_issued = floor(circulating_supply * effective_rate / epochs_per_year)`
- 增发分配：
  - `staking = epoch_issued * staking_reward_bps / 10000`
  - `node_service = epoch_issued * node_service_reward_bps / 10000`
  - `ecosystem = epoch_issued * ecosystem_pool_bps / 10000`
  - `security_reserve = remainder`

### 4) 动作与事件（草案）
```rust
Action::InitializeMainTokenGenesis { allocations: Vec<GenesisAllocationBucket> }
Action::ClaimMainTokenVesting { bucket_id: String, beneficiary: String, nonce: u64 }
Action::ApplyEpochIssuance { epoch_index: u64, actual_stake_ratio_bps: u32 }
Action::UpdateMainTokenPolicy { proposal_id: String, next: MainTokenConfig }

DomainEvent::MainTokenGenesisInitialized { total_supply: u128, bucket_count: usize }
DomainEvent::MainTokenVestingClaimed { bucket_id: String, beneficiary: String, amount: u128 }
DomainEvent::MainTokenEpochIssued { epoch_index: u64, issued: u128, rate_bps: u32 }
DomainEvent::MainTokenBurned { reason: String, amount: u128 }
DomainEvent::MainTokenPolicyUpdated { proposal_id: String, effective_epoch: u64 }
```

### 5) 核心不变量（草案）
- `sum(bucket.ratio_bps) == 10000`
- `sum(genesis_minted) == initial_supply`
- `claimed_vesting <= releasable_vesting`
- `effective_rate_bps` 必须在 `[min_rate_bps, max_rate_bps]`
- `sum(split_bps) == 10000`
- `total_supply = initial_supply + total_issued - total_burned`

### 6) 与现有奖励系统关系（草案）
- 现有 `NodePoints/PowerCredit` 保持独立运行，不直接替代主链原生 Token。
- 增发分配中的 `node_service_reward_bps` 通过桥接动作消费 `EpochSettlementReport` 聚合值，作为 AWT 侧奖励输入。
- 未启用桥接时，`node_service_reward_bps` 可暂时进入 `protocol_treasury`，避免旁路增发。

## 里程碑
- **TAM-M0**：设计文档 + 项目管理文档建档。
- **TAM-M1**：主链 Token 状态模型与快照字段落地。
- **TAM-M2**：创世分配、解锁领取与审计事件落地。
- **TAM-M3**：epoch 增发与分配执行路径落地。
- **TAM-M4**：销毁/罚没/金库记账闭环落地。
- **TAM-M5**：治理更新边界与生效延迟落地。
- **TAM-M6**：NodePoints 桥接接口接线与回归。
- **TAM-M7**：`test_tier_required/full` 测试矩阵与发布文档收口。

## 测试策略
- `test_tier_required`：
  - 创世分配比例守恒；
  - vesting 领取金额与 epoch 线性解锁正确；
  - 通胀率边界 clamp 正确；
  - 分配比例守恒与余数归并确定性；
  - policy 越界更新被拒绝。
- `test_tier_full`：
  - 多 epoch 增发与销毁后总量守恒；
  - 快照恢复后供应量与各桶余额一致；
  - NodePoints 桥接启停切换下的分配一致性；
  - 治理参数变更前后回放一致性。

## 风险
- 参数风险：初始分配与通胀区间设置不当会造成早期过度稀释或激励不足。
- 中心化风险：大比例金库或贡献者配额若缺少治理约束，可能引发控制权集中。
- 实施风险：主链 Token 与现有 `PowerCredit` 并行阶段易出现“重复激励”误配。
- 迁移风险：历史快照未包含主链 token 字段时，需要严格兼容默认值与版本迁移。
