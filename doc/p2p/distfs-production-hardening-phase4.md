# Agent World Runtime：DistFS 生产化增强（Phase 4）设计文档

## 目标
- 在 DistFS 挑战探测链路引入“有状态调度”，避免每轮重复命中同一批 blob，提高挑战覆盖率与公平性。
- 为 reward runtime 增加挑战调度状态持久化与恢复能力，保证重启后探测序列连续。
- 保持现有 reward settlement 主链路兼容：挑战路径异常时不阻断结算。

## 范围

### In Scope
- **DPH4-1：文档与任务拆解**
  - 输出 Phase 4 设计文档与项目管理文档。

- **DPH4-2：DistFS 有状态挑战调度接口**
  - 新增调度状态模型（cursor + 累计统计）。
  - 新增 `LocalCasStore::probe_storage_challenges_with_cursor(...)`：
    - 按 cursor 轮转选择 blob；
    - 每轮挑战后推进 cursor；
    - 累计挑战统计与失败原因。
  - 补充单元测试（轮转推进、失败分类、空集路径）。

- **DPH4-3：reward runtime 持久化接线**
  - `world_viewer_live` reward runtime 启动时加载 DistFS probe 状态。
  - 运行中每轮探测后持久化状态，重启可恢复。
  - 补充 `world_viewer_live` 测试（状态 roundtrip、probe 调用路径）。

- **DPH4-4：回归与收口**
  - 执行 `agent_world_distfs` 与 `world_viewer_live` 回归。
  - 回写项目状态与 devlog。

### Out of Scope
- 跨节点统一挑战调度器（网络级 coordinator）。
- 链上惩罚执行与治理参数自动调整。
- ZK/PoRep/PoSt 证明协议升级。

## 接口 / 数据

### 调度状态（草案）
```rust
StorageChallengeProbeCursorState {
  next_blob_cursor: usize,
  rounds_executed: u64,
  cumulative_total_checks: u64,
  cumulative_passed_checks: u64,
  cumulative_failed_checks: u64,
  cumulative_failure_reasons: BTreeMap<String, u64>,
}
```

### 有状态探测（草案）
```rust
LocalCasStore::probe_storage_challenges_with_cursor(
  world_id: &str,
  node_id: &str,
  observed_at_unix_ms: i64,
  config: &StorageChallengeProbeConfig,
  state: &mut StorageChallengeProbeCursorState,
) -> Result<StorageChallengeProbeReport, WorldError>
```

### reward runtime 接线（草案）
- 新增状态文件：`reward-runtime-distfs-probe-state.json`。
- 生命周期：启动加载 -> 每轮更新 -> 原子写回。

## 里程碑
- **DPH4-M1**：文档与任务拆解完成。
- **DPH4-M2**：DistFS 有状态探测能力完成。
- **DPH4-M3**：reward runtime 状态持久化接线完成。
- **DPH4-M4**：回归与文档收口完成。

## 风险
- 若 blob 集合动态变化较快，cursor 可能短期跳跃；通过 `% blob_count` 约束和累计统计降低影响。
- 状态文件损坏会导致探测重置；通过容错加载（失败回退默认状态）保证服务可用。
- 高频持久化增加 I/O；当前每轮一写可接受，后续可加节流。
