# Agent World Runtime：节点基础在线时长奖励（设计文档）

## 目标
- 在现有节点贡献积分（Node Points）中加入“基础在线时长奖励”可运行实现。
- 在线奖励基于“挑战通过率”而不是节点自报时长，降低假在线收益。
- 保持与已有计算/存储/可靠性积分模型兼容，不破坏现有结算台账结构。

## 范围

### In Scope
- 在 `NodePointsConfig` 增加在线奖励门槛参数：`min_uptime_challenge_pass_ratio`。
- 在 `NodeContributionSample` 增加在线挑战统计：
  - `uptime_valid_checks`
  - `uptime_total_checks`
- 在结算逻辑中引入在线得分归一化：
  - `raw_uptime_ratio = valid / total`（当 `total > 0`）
  - 否则 fallback 到 `uptime_seconds / epoch_duration_seconds`
  - `uptime_score = max(0, (raw_uptime_ratio - min_ratio) / (1 - min_ratio))`
- 在 runtime collector 中接入挑战统计采样，并将样本传入 settlement。
- 单元测试覆盖：门槛生效、fallback、生效后奖励差异、runtime 采样接线。

### Out of Scope
- 链上随机挑战（VRF）和跨节点挑战网络协议。
- 罚没资金清算与仲裁系统。
- 多观察点拜占庭容错（本次仅做基础版采样结构）。

## 接口 / 数据

### 配置新增字段
```rust
NodePointsConfig {
  // 0.0..=1.0，低于该挑战通过率时在线项不计分
  min_uptime_challenge_pass_ratio: f64,
}
```

### 贡献样本新增字段
```rust
NodeContributionSample {
  // 当 epoch 内有挑战记录时，在线得分以挑战统计为准
  uptime_valid_checks: u64,
  uptime_total_checks: u64,
}
```

### Runtime 观察新增字段
```rust
NodePointsRuntimeObservation {
  uptime_checks_passed: u64,
  uptime_checks_total: u64,
}
```

### 结算规则（在线项）
- 若 `uptime_total_checks > 0`，在线率使用挑战通过率；
- 否则使用 `uptime_seconds / epoch_duration_seconds` 作为回退口径；
- 在线奖励门槛通过 `min_uptime_challenge_pass_ratio` 控制，低于门槛不给在线分；
- 达标后按线性归一化给分，最大不超过 1。

## 里程碑
- UBR-1：完成设计文档与项目管理文档。
- UBR-2：完成 `node_points` 在线挑战奖励实现与测试。
- UBR-3：完成 `node_points_runtime` 挑战采样接线与测试。
- UBR-4：完成回归测试、文档状态回写和 devlog 收口。

## 风险
- 若挑战频率过低，在线率统计波动会偏大；需后续引入最小挑战数门槛。
- 目前默认采样仍可被本地进程状态影响，后续应接入多观察点挑战源。
- 配置 `min_uptime_challenge_pass_ratio` 过高可能导致新节点难以拿到在线分，需结合运营参数调优。
