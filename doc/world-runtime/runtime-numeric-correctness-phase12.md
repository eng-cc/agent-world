# Agent World Runtime：Membership Recovery 调度门控与计数聚合算术语义硬化（15 点清单第十二阶段）

## 目标
- 收口 `membership_recovery/mod.rs` 中仍使用饱和算术的调度门控与关键计数聚合路径。
- 在极端时间边界与计数边界下，将“静默饱和继续执行”升级为“显式失败且不污染状态”。
- 保持 recovery/dead-letter 写入原子性：失败路径不更新 `last_replay_at_ms`、不写入部分 pending/dead-letter 结果。

## 范围

### In Scope（第十二阶段）
- `crates/agent_world_consensus/src/membership_recovery/mod.rs`
  - `run_revocation_dead_letter_replay_schedule` 的 interval gate 时间差从 `saturating_sub` 改为受检减法。
  - `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter` 的关键容量/计数累加从饱和语义改为受检语义（越界显式失败）。
  - 统一溢出错误语义为 `WorldError::DistributedValidationFailed`，错误消息包含关键现场值。
- 测试
  - 在 `membership_recovery_tests.rs` 增加时间差溢出拒绝测试与计数边界拒绝测试。
  - 验证失败路径不写 `last_replay_at_ms`，不提交部分 recovery/dead-letter 状态。

### Out of Scope（后续阶段）
- `membership_recovery/replay.rs` 之外其他 replay 子模块（archive/federated/audit）剩余饱和计数收口。
- `membership.rs`、`mempool.rs` 等非本阶段目标文件的全量计数语义统一。
- 全链路数值 newtype 与跨模块统一时钟治理。

## 接口/数据
- 对外 API 入口保持不变（继续返回 `Result<..., WorldError>`）。
- 内部将新增/复用受检加法 helper，替换关键 `saturating_add`。
- 失败模型统一：
  - `WorldError::DistributedValidationFailed`
  - 错误消息包含 `now_ms/last_replay_at_ms` 或计数字段上下文。

## 里程碑
- M0：Phase12 建档并冻结边界。
- M1：replay schedule 时间门控受检语义改造完成。
- M2：recovery 计数/容量受检语义改造与边界测试完成。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- 从饱和语义转为显式失败后，历史“边界夹逼继续执行”路径会变为拒绝，需要同步更新测试预期。
- 计数路径分支较多，若改造不一致，可能导致 report/metrics 与落盘状态不一致。
- 需重点验证失败发生在状态写入前，避免 pending/dead-letter 半更新。

## 当前状态
- 截至 2026-02-23：M0 已完成；M1、M2、M3 待执行。
