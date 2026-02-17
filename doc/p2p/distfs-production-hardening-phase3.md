# Agent World Runtime：DistFS 生产化增强（Phase 3）设计文档

## 目标
- 将 DistFS 挑战机制从“库内能力”推进到“reward runtime 可消费”的生产闭环。
- 让 `storage_valid_checks/storage_total_checks` 使用真实挑战结果，而非固定启发式计数。
- 保持兼容：无可挑战数据时不阻断结算主链路。

## 范围

### In Scope
- **DPH3-1：文档与任务拆解**
  - 输出 Phase 3 设计文档与项目管理文档。

- **DPH3-2：DistFS 挑战探测接口（Probe）**
  - 在 `agent_world_distfs` 增加“按 tick 发起挑战并汇总统计”的统一入口。
  - 输出节点维度挑战统计（total/pass/fail + 失败原因分布）。

- **DPH3-3：reward runtime 接线真实挑战计数**
  - 在 `world_viewer_live` 的 reward runtime 循环中引入 DistFS probe。
  - 以 probe 结果覆盖 observation 的 `storage_checks_passed/total`。
  - 将挑战统计写入 epoch 报告，提升可观测性。

- **DPH3-4：回归与收口**
  - 完成 `agent_world_distfs` 与 `world_viewer_live` 相关测试回归。
  - 回写项目文档状态与 devlog。

### Out of Scope
- 多节点网络挑战调度器。
- 链上罚没执行与治理参数上链。
- ZK/PoRep/PoSt 证明系统接入。

## 接口 / 数据

### DistFS Probe（草案）
```rust
StorageChallengeProbeConfig {
  max_sample_bytes: u32,
  challenges_per_tick: u32,
  challenge_ttl_ms: i64,
  allowed_clock_skew_ms: i64,
}

StorageChallengeProbeReport {
  node_id: String,
  world_id: String,
  observed_at_unix_ms: i64,
  total_checks: u64,
  passed_checks: u64,
  failed_checks: u64,
  failure_reasons: BTreeMap<String, u64>,
  latest_proof_semantics: Option<StorageChallengeProofSemantics>,
}

LocalCasStore::probe_storage_challenges(
  world_id: &str,
  node_id: &str,
  observed_at_unix_ms: i64,
  config: &StorageChallengeProbeConfig,
) -> Result<StorageChallengeProbeReport, WorldError>
```

### Reward Runtime 接线（草案）
- 每轮 `reward_runtime_loop`：
  - 对 `storage_root` 执行 probe；
  - 将 `passed/total` 注入 `NodePointsRuntimeObservation`；
  - 把 probe 报告写入 epoch JSON 报告字段（`distfs_challenge_report`）。

### 兼容语义
- 若 probe 失败或无样本，保留既有 observation 逻辑，不阻断 reward settlement。

## 里程碑
- **DPH3-M1**：文档与任务拆解完成。
- **DPH3-M2**：DistFS Probe 能力与单测完成。
- **DPH3-M3**：reward runtime 接线与单测完成。
- **DPH3-M4**：回归与文档收口完成。

## 风险
- 挑战频率过高会增加 I/O 压力；需默认保守参数（低频小样本）。
- 节点无 blob 时统计为 0，可能导致短期奖励偏低；需在配置侧配合挑战调度策略。
- 回执失败原因分类若过粗，会降低运维定位效率；本期先提供稳定枚举映射，后续细化。
