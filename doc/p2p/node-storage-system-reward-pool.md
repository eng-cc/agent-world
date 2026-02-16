# Agent World Runtime：节点存储系统奖励池（设计文档）

## 目标
- 在 Node Points 结算中新增“存储系统奖励池”，使存储奖励和在线奖励一样由协议固定池发放。
- 存储奖励以挑战通过率为核心资格，避免仅靠自报容量领取奖励。
- 在保持现有结算接口可兼容的前提下，提供生产可用的可观测字段（主池/存储池拆分）。

## 范围

### In Scope
- 在 `NodePointsConfig` 新增存储池与反作弊参数：
  - `storage_pool_points`
  - `min_storage_challenge_pass_ratio`
  - `min_storage_challenge_checks`
  - `max_rewardable_storage_to_staked_ratio`（0 表示关闭按质押封顶）
- 在 `NodeContributionSample` 新增存储挑战/质押字段：
  - `storage_valid_checks`
  - `storage_total_checks`
  - `staked_storage_bytes`
- 在 `NodeSettlement`/`EpochSettlementReport` 新增拆池观测字段：
  - 主池积分、存储池积分、总积分；
  - 存储奖励评分、可奖励存储字节。
- 结算逻辑采用“双池分配”：
  - 主池沿用既有多维评分（compute/storage/uptime/reliability - penalty）；
  - 存储池按独立评分分配，并累加到总奖励。
- 单元测试覆盖：
  - 存储池独立分配守恒；
  - 挑战门槛和最小挑战数生效；
  - 质押封顶生效；
  - 主池与存储池叠加后累计积分正确。

### Out of Scope
- 链上质押资产真实扣罚（本次仅接入样本字段和封顶约束）。
- 跨节点挑战协议、VRF 随机挑战网络实现。
- 需求侧订单支付市场（本次聚焦系统奖励池）。

## 接口 / 数据

### 配置新增字段
```rust
NodePointsConfig {
  storage_pool_points: u64,
  min_storage_challenge_pass_ratio: f64,
  min_storage_challenge_checks: u64,
  // 可奖励存储上限 = staked_storage_bytes * ratio；0 表示不封顶
  max_rewardable_storage_to_staked_ratio: f64,
}
```

### 样本新增字段
```rust
NodeContributionSample {
  storage_valid_checks: u64,
  storage_total_checks: u64,
  staked_storage_bytes: u64,
}
```

### 结算新增字段
```rust
NodeSettlement {
  storage_reward_score: f64,
  rewardable_storage_bytes: u64,
  main_awarded_points: u64,
  storage_awarded_points: u64,
  awarded_points: u64, // 总奖励
}

EpochSettlementReport {
  pool_points: u64,            // 主池
  storage_pool_points: u64,    // 存储池
  distributed_points: u64,     // 主池已分配
  storage_distributed_points: u64,
  total_distributed_points: u64,
}
```

### 结算规则（存储池）
- 挑战通过率：`storage_pass_ratio = valid / total`（`total > 0`）。
- 资格门槛：
  - `total >= min_storage_challenge_checks`；
  - `storage_pass_ratio > min_storage_challenge_pass_ratio`。
- 通过率归一化：
  - `norm = max(0, (pass - min_ratio) / (1 - min_ratio))`。
- 可奖励存储：
  - 默认 `rewardable = effective_storage_bytes`；
  - 若 `max_rewardable_storage_to_staked_ratio > 0` 且 `staked_storage_bytes > 0`，
    `rewardable = min(effective_storage_bytes, staked_storage_bytes * ratio)`。
- 存储池评分：
  - `storage_reward_score = sqrt(rewardable_gib) * norm * availability_ratio`。

## 里程碑
- SBR-1：设计文档与项目管理文档。
- SBR-2：`node_points` 双池结算与测试。
- SBR-3：`node_points_runtime` 存储挑战采样接线与测试。
- SBR-4：`test_tier_required` 回归、文档与 devlog 收口。

## 风险
- 若挑战次数不足，存储池分配可能频繁空池；需后续配合挑战调度频率。
- 质押封顶参数配置不当会抑制真实大节点贡献，需按网络规模调优。
- 主池仍包含 `weight_storage` 时可能与存储池形成叠加激励，部署时应结合参数策略（例如将主池 `weight_storage` 降低或置零）。
