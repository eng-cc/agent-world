# Agent World Runtime：节点奖励运行时生产化加固（Phase 1）设计文档

## 目标
- 将现有奖励链路从“演示可跑”提升为“可持续运行、可恢复、可审计”的生产化基础能力。
- 补齐奖励运行时状态持久化，避免进程重启导致 epoch/累计积分/采样窗口丢失。
- 收口兑换签名授权语义，支持策略化强制 `signer_node_id == node_id`，防止托管式滥签默认放开。
- 移除 reward runtime 中的占位身份绑定行为，改为显式绑定来源，避免伪身份进入账本。

## 范围

### In Scope
- **PRH1-1：奖励采样状态持久化**
  - 为 `NodePointsLedger` 增加可序列化快照结构。
  - 为 `NodePointsRuntimeCollector` 增加快照导出/恢复能力。
  - 为 `world_viewer_live` reward runtime 增加状态文件落盘与启动恢复。

- **PRH1-2：兑换签名授权收口**
  - 在 `RewardSignatureGovernancePolicy` 新增 `require_redeem_signer_match_node_id`。
  - 兑换校验路径支持策略化强制“签名节点必须等于被兑换节点”。
  - 保持默认兼容（不强制），生产路径显式开启。

- **PRH1-3：身份绑定行为加固**
  - 删除 reward runtime 对 settlement 节点的占位公钥自动绑定。
  - reward runtime 仅绑定明确来源身份（signer/node），出现缺失时明确报错。

- **PRH1-4：回归测试与可观测性**
  - 新增/更新单元测试：collector 快照恢复、策略门禁、runtime 参数默认值。
  - 维持 `test_tier_required` 口径回归通过。

### Out of Scope
- 完整多节点链上奖励结算调度器（独立可执行进程与网络共识提案）。
- 真实 PoRep/PoSt/VRF 挑战协议及跨观察点拜占庭验证。
- 多签/HSM/KMS 托管体系与跨组织授权治理。
- 需求侧订单撮合市场和动态价格机制。

## 接口 / 数据

### 1) `RewardSignatureGovernancePolicy` 新增字段
```rust
RewardSignatureGovernancePolicy {
  require_mintsig_v2: bool,
  allow_mintsig_v1_fallback: bool,
  require_redeem_signature: bool,
  // true 时强制 signer_node_id == node_id
  require_redeem_signer_match_node_id: bool,
}
```

### 2) Node Points 账本快照（新增）
```rust
NodePointsLedgerSnapshot {
  config: NodePointsConfig,
  epoch_index: u64,
  cumulative_points: BTreeMap<String, u64>,
}
```

### 3) Collector 快照（新增）
```rust
NodePointsRuntimeCollectorSnapshot {
  ledger: NodePointsLedgerSnapshot,
  heuristics: NodePointsRuntimeHeuristics,
  epoch_started_at_unix_ms: Option<i64>,
  cursors: BTreeMap<String, NodeCursor>,
  current_epoch: BTreeMap<String, NodeEpochAccumulator>,
}
```

### 4) Reward Runtime 状态文件（新增）
- 路径：`<reward_runtime_report_dir>/reward-runtime-state.json`
- 内容：`NodePointsRuntimeCollectorSnapshot`。
- 语义：启动优先加载，运行中按固定频率（每次采样）原子写入。

## 里程碑
- **PRH1-M1**：设计/项目文档落地。
- **PRH1-M2**：collector/ledger 快照序列化与恢复实现。
- **PRH1-M3**：兑换签名授权策略字段 + 校验门禁实现。
- **PRH1-M4**：reward runtime 身份绑定行为收口。
- **PRH1-M5**：测试回归、文档状态更新、devlog 收口。

## 风险
- 状态文件损坏可能导致 runtime 恢复失败，需要安全回退为“空状态启动”。
- 强制 `signer==node` 策略开启后，历史托管签名流程将被拒绝，需要灰度切换。
- 高频状态落盘会增加 I/O 压力，Phase 1 先保证一致性，后续再做节流优化。
