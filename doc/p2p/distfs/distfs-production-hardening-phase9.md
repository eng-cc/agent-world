# Agent World Runtime：DistFS 生产化增强（Phase 9）设计文档

## 目标
- 为 DistFS 自适应挑战调度补充退避决策级别可观测数据（backoff observability），便于生产排障与参数调优。
- 在保持向后兼容的前提下扩展 probe cursor 状态结构，保留旧状态文件可恢复能力。
- 将新增观测字段接入现有测试闭环，确保 runtime 持久化与报告链路可稳定消费。

## 范围

### In Scope
- **DPH9-1：文档与任务拆解**
  - 输出 Phase 9 设计文档与项目管理文档。

- **DPH9-2：调度状态观测字段扩展（DistFS）**
  - 扩展 `StorageChallengeProbeCursorState`，新增退避观测字段：
    - `cumulative_backoff_skipped_rounds`
    - `cumulative_backoff_applied_ms`
    - `last_backoff_duration_ms`
    - `last_backoff_reason`
    - `last_backoff_multiplier`
  - 在调度流程中按真实行为更新上述字段（执行退避、因退避跳过）。
  - 新字段均使用 `serde(default)` 保持旧状态快照兼容。

- **DPH9-3：Runtime 序列化与单测覆盖增强**
  - 补齐 `world_viewer_live` 相关测试：
    - DistFS probe state roundtrip 包含新增字段；
    - 报告序列化含新增字段；
    - 关键退避路径行为断言。

- **DPH9-4：回归与收口**
  - 执行 DistFS 与 `world_viewer_live` 回归。
  - 回写项目状态与 devlog。

### Out of Scope
- 新增 reward 结算公式。
- 引入新的 challenge 类型或证明系统。
- 分布式集中调度器（orchestrator）。

## 接口 / 数据

### 状态结构扩展（草案）
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
  cumulative_backoff_skipped_rounds: u64,
  cumulative_backoff_applied_ms: i64,
  last_backoff_duration_ms: i64,
  last_backoff_reason: Option<String>,
  last_backoff_multiplier: u32,
}
```

## 里程碑
- **DPH9-M1**：文档与任务拆解完成。
- **DPH9-M2**：调度状态扩展与行为接线完成。
- **DPH9-M3**：runtime 单测与序列化覆盖完成。
- **DPH9-M4**：回归与文档收口完成。

## 风险
- 状态字段增长可能带来维护成本；通过命名约束与测试覆盖控制。
- 观测字段若更新时机不一致会误导运维；通过行为单测锁定语义。
- 回写逻辑若处理不当可能破坏兼容；通过 `serde(default)` 与 legacy 反序列化测试兜底。
