# Agent World Runtime：节点贡献积分激励（设计文档）

## 目标
- 在 Agent World 的区块链 + P2P FS 闭环内，引入可审计的节点积分激励（Node Points）。
- 明确“基础义务”和“额外贡献”的边界：
  - 为自身 Agent 提供模拟计算属于基础义务，不直接奖励；
  - 为离线节点代跑模拟、执行世界维护任务属于额外计算，应获得奖励。
- 为长期在线且提供更多有效存储的节点提供额外收益。
- 前期收益形态采用积分（不可转账），后续再映射到模拟内资源/货币。

## 范围

### In Scope
- 新增节点积分结算引擎（epoch 级）。
- 贡献维度：
  - `delegated_sim_compute_units`（代跑离线节点）；
  - `world_maintenance_compute_units`（世界维护任务）；
  - `effective_storage_bytes`（有效存储）；
  - `uptime_seconds`（在线时长）；
  - `verify_pass_ratio` / `availability_ratio`（可靠性）。
- 基础义务校验：`self_sim_compute_units` 仅用于最低义务判断，不产生奖励。
- 结算输出：每 epoch 节点积分、累计积分台账、惩罚明细。
- 单元测试覆盖：权重计算、惩罚、积分池守恒、跨 epoch 累计。

### Out of Scope
- 链上可交易代币、真实经济清算。
- 完整质押/罚没资产系统（仅保留积分惩罚入口）。
- 复杂证明协议（PoRep/PoSt/ZK）的真实网络接线。

## 接口 / 数据

### 核心配置（草案）
```rust
NodePointsConfig {
  epoch_duration_seconds: u64,
  epoch_pool_points: u64,
  min_self_sim_compute_units: u64,
  delegated_compute_multiplier: f64,
  maintenance_compute_multiplier: f64,
  weight_compute: f64,
  weight_storage: f64,
  weight_uptime: f64,
  weight_reliability: f64,
  obligation_penalty_points: f64,
}
```

### 节点贡献输入（草案）
```rust
NodeContributionSample {
  node_id: String,
  self_sim_compute_units: u64,
  delegated_sim_compute_units: u64,
  world_maintenance_compute_units: u64,
  effective_storage_bytes: u64,
  uptime_seconds: u64,
  verify_pass_ratio: f64,
  availability_ratio: f64,
  explicit_penalty_points: f64,
}
```

### 结算输出（草案）
```rust
NodeSettlement {
  node_id: String,
  obligation_met: bool,
  compute_score: f64,
  storage_score: f64,
  uptime_score: f64,
  reliability_score: f64,
  penalty_score: f64,
  total_score: f64,
  awarded_points: u64,
  cumulative_points: u64,
}

EpochSettlementReport {
  epoch_index: u64,
  pool_points: u64,
  distributed_points: u64,
  settlements: Vec<NodeSettlement>,
}
```

### 计分公式（MVP）
- 额外计算分：
  - `compute_units = delegated * delegated_multiplier + maintenance * maintenance_multiplier`
  - `compute_score = compute_units * verify_pass_ratio`
- 存储分：
  - `storage_gib = effective_storage_bytes / 1024^3`
  - `storage_score = sqrt(storage_gib) * availability_ratio`
- 在线分：
  - `uptime_score = min(1.0, uptime_seconds / epoch_duration_seconds)`
- 可靠性分：
  - `reliability_score = (verify_pass_ratio + availability_ratio) / 2`
- 总分：
  - `total = w_c*compute + w_s*storage + w_u*uptime + w_r*reliability - penalty`
  - `total < 0` 则按 `0` 处理。
- 基础义务惩罚：
  - 当 `self_sim_compute_units < min_self_sim_compute_units` 时，额外加罚 `obligation_penalty_points`。

## 里程碑
- NCP-1：设计文档 + 项目管理文档。
- NCP-2：节点积分引擎核心实现（计算/存储/在线/惩罚 + 台账）。
- NCP-3：测试与导出接线（test_tier_required 口径）。
- NCP-4：文档状态回写与 devlog 收口。

## 风险
- 参数不当可能导致单一资源（大存储或大算力）垄断积分，需要通过 `sqrt(storage)` 与权重平衡缓解。
- 若没有真实证明接线，`verify_pass_ratio/availability_ratio` 的真实性依赖上层采样器，后续需替换为链路证明数据。
- 积分池固定时，低活跃 epoch 可能出现“有效贡献过少”，需在后续迭代加入最小活跃阈值与回收池机制。
