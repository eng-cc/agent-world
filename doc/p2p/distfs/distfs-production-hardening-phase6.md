# Agent World Runtime：DistFS 生产化增强（Phase 6）设计文档

## 目标
- 为 DistFS 有状态挑战调度增加自适应失败退避（backoff）能力，减少持续失败时的 I/O 抖动与无效探测。
- 引入每轮挑战预算上限，避免在高密度 blob 场景下单轮探测放大。
- 保持历史状态文件兼容，确保线上升级不会因状态字段演进而启动失败。

## 范围

### In Scope
- **DPH6-1：文档与任务拆解**
  - 输出 Phase 6 设计文档与项目管理文档。

- **DPH6-2：自适应策略与预算上限**
  - 在 `challenge_scheduler` 新增自适应调度策略结构。
  - 新增 `probe_storage_challenges_with_policy(...)`：
    - 按策略限制单轮挑战数量；
    - 根据连续失败轮次执行指数退避；
    - 退避窗口内跳过探测。
  - 保留 `probe_storage_challenges_with_cursor(...)` 兼容入口（默认策略）。

- **DPH6-3：状态演进与兼容**
  - 扩展 `StorageChallengeProbeCursorState`：新增连续失败/退避截止/最后探测时刻字段。
  - 使用 `serde(default)` 保证旧状态文件可反序列化。
  - 增加状态兼容与策略行为单测。

- **DPH6-4：回归与收口**
  - 执行 `agent_world_distfs` 与 `world_viewer_live` 相关回归。
  - 回写项目状态与 devlog。

### Out of Scope
- 多节点统一协调器与跨节点预算仲裁。
- 链上动态参数治理。
- ZK/PoRep/PoSt 升级。

## 接口 / 数据

### 自适应策略（草案）
```rust
StorageChallengeAdaptivePolicy {
  max_checks_per_round: u32,
  failure_backoff_base_ms: i64,
  failure_backoff_max_ms: i64,
}
```

### 状态扩展（草案）
```rust
StorageChallengeProbeCursorState {
  next_blob_cursor: usize,
  rounds_executed: u64,
  cumulative_total_checks: u64,
  cumulative_passed_checks: u64,
  cumulative_failed_checks: u64,
  cumulative_failure_reasons: BTreeMap<String, u64>,
  consecutive_failure_rounds: u64,
  backoff_until_unix_ms: i64,
  last_probe_unix_ms: Option<i64>,
}
```

## 里程碑
- **DPH6-M1**：文档与任务拆解完成。
- **DPH6-M2**：自适应策略与预算上限完成。
- **DPH6-M3**：状态兼容与测试完成。
- **DPH6-M4**：回归与文档收口完成。

## 风险
- 退避配置不合理可能导致探测过稀；通过默认策略保持“退避关闭”兼容。
- 状态字段扩展若无默认值会破坏旧状态恢复；通过 `serde(default)` 强约束。
- 预算限制过低会影响挑战覆盖率；后续可通过 Phase 5 CLI 参数联动调优。
