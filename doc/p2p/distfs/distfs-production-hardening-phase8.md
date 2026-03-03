# Agent World Runtime：DistFS 生产化增强（Phase 8）设计文档

## 目标
- 将 reason-aware 退避策略中的失败原因倍率参数（multiplier）完整治理化到 `world_viewer_live` CLI。
- 让 reward runtime 运行时可按失败原因分层调优退避，不再依赖编译期默认值。
- 保持主入口文件规模可控并补齐参数解析与序列化观测测试。

## 范围

### In Scope
- **DPH8-1：文档与任务拆解**
  - 输出 Phase 8 设计文档与项目管理文档。

- **DPH8-2：Multiplier 参数治理化（CLI + Runtime）**
  - 为如下字段新增 CLI 参数并接入 `DistfsProbeRuntimeConfig.adaptive_policy`：
    - `backoff_multiplier_hash_mismatch`
    - `backoff_multiplier_missing_sample`
    - `backoff_multiplier_timeout`
    - `backoff_multiplier_read_io_error`
    - `backoff_multiplier_signature_invalid`
    - `backoff_multiplier_unknown`
  - 参数校验：均为 `u32` 且 `>= 1`。

- **DPH8-3：测试与可观测覆盖增强**
  - 增补 `world_viewer_live` 参数解析测试（自定义值 + 非法值）。
  - 增补 runtime 配置序列化断言，确保 multiplier 可在报告链路观测。

- **DPH8-4：回归与收口**
  - 执行 DistFS 与 world viewer 相关回归。
  - 回写项目文档与 devlog。

### Out of Scope
- 动态链上配置下发 multiplier。
- 多策略模板自动切换。
- challenge 类型扩展（如 PoRep/PoSt 新证明类型）。

## 接口 / 数据

### 新增 CLI 参数（草案）
```text
--reward-distfs-adaptive-multiplier-hash-mismatch <u32, >=1>
--reward-distfs-adaptive-multiplier-missing-sample <u32, >=1>
--reward-distfs-adaptive-multiplier-timeout <u32, >=1>
--reward-distfs-adaptive-multiplier-read-io-error <u32, >=1>
--reward-distfs-adaptive-multiplier-signature-invalid <u32, >=1>
--reward-distfs-adaptive-multiplier-unknown <u32, >=1>
```

### 配置结构（延续）
```rust
DistfsProbeRuntimeConfig {
  max_sample_bytes: u32,
  challenges_per_tick: u32,
  challenge_ttl_ms: i64,
  allowed_clock_skew_ms: i64,
  adaptive_policy: StorageChallengeAdaptivePolicy,
}
```

## 里程碑
- **DPH8-M1**：文档与任务拆解完成。
- **DPH8-M2**：CLI multiplier 参数接线完成。
- **DPH8-M3**：测试与可观测覆盖完成。
- **DPH8-M4**：回归与文档收口完成。

## 风险
- 参数数量继续增加，运维复杂度上升；通过默认值与严格参数校验控制风险。
- 倍率配置过高可能带来过度退避；通过 `backoff_max_ms` 上限兜底。
- 主入口帮助文本持续膨胀；通过解析逻辑模块化和测试覆盖控制维护成本。
