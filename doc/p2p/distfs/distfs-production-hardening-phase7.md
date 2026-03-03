# Agent World Runtime：DistFS 生产化增强（Phase 7）设计文档

## 目标
- 在 DistFS 自适应挑战调度中引入按失败原因分级退避（reason-aware backoff），提高失败处理精度。
- 将自适应调度参数治理化到 `world_viewer_live` CLI，支持运行时调优而无需改代码。
- 在不突破单文件行数约束的前提下继续模块化 `world_viewer_live`。

## 范围

### In Scope
- **DPH7-1：文档与任务拆解**
  - 输出 Phase 7 设计文档与项目管理文档。

- **DPH7-2：Reason-aware 退避策略（DistFS）**
  - 扩展 `StorageChallengeAdaptivePolicy`：
    - `backoff_multiplier_hash_mismatch`;
    - `backoff_multiplier_missing_sample`;
    - `backoff_multiplier_timeout`;
    - `backoff_multiplier_read_io_error`;
    - `backoff_multiplier_signature_invalid`;
    - `backoff_multiplier_unknown`。
  - 退避计算按当轮主失败原因选择倍率。
  - 补齐单测（不同失败原因触发不同 backoff）。

- **DPH7-3：CLI 参数治理化 + 模块化**
  - 在 reward runtime 增加自适应策略配置（持有于 runtime config）。
  - 新增 CLI 参数：
    - `--reward-distfs-adaptive-max-checks-per-round`
    - `--reward-distfs-adaptive-backoff-base-ms`
    - `--reward-distfs-adaptive-backoff-max-ms`
  - `world_viewer_live` 调用 probe 时传入治理化策略。
  - 必要时继续拆分 `world_viewer_live` 辅助函数到子模块，保证单文件 <=1200 行。
  - 补齐参数解析与接线测试。

- **DPH7-4：回归与收口**
  - 执行 DistFS 与 world_viewer_live 回归。
  - 回写项目状态与 devlog。

### Out of Scope
- 链上治理参数自动拉取。
- 多节点集中式 challenge orchestrator。
- PoRep/PoSt 升级。

## 接口 / 数据

### 策略扩展（草案）
```rust
StorageChallengeAdaptivePolicy {
  max_checks_per_round: u32,
  failure_backoff_base_ms: i64,
  failure_backoff_max_ms: i64,
  backoff_multiplier_hash_mismatch: u32,
  backoff_multiplier_missing_sample: u32,
  backoff_multiplier_timeout: u32,
  backoff_multiplier_read_io_error: u32,
  backoff_multiplier_signature_invalid: u32,
  backoff_multiplier_unknown: u32,
}
```

### CLI 参数（草案）
```text
--reward-distfs-adaptive-max-checks-per-round <u32, >0>
--reward-distfs-adaptive-backoff-base-ms <i64, >=0>
--reward-distfs-adaptive-backoff-max-ms <i64, >=0, >=base>
```

## 里程碑
- **DPH7-M1**：文档与任务拆解完成。
- **DPH7-M2**：reason-aware 退避策略完成。
- **DPH7-M3**：CLI 参数治理化与接线完成。
- **DPH7-M4**：回归与文档收口完成。

## 风险
- 参数组合过多会增加运维配置复杂度；通过默认策略与严格校验降低误配风险。
- reason 分类若不稳定可能造成退避抖动；本期沿用稳定失败原因枚举。
- CLI 扩展可能推高主文件复杂度；通过模块化拆分控制技术债。
