# Agent World Runtime：Membership Recovery Federated 聚合扫描与游标计数算术语义硬化（15 点清单第十五阶段）

## 目标
- 收口 `membership_recovery/replay_archive_federated.rs` 中剩余的关键聚合计数饱和算术。
- 在极端计数边界下，将“静默饱和继续执行”升级为“显式失败且不污染查询状态”。
- 完成 15 点清单最后阶段的数值语义收口，形成一致的受检算术策略。

## 范围

### In Scope（第十五阶段）
- `crates/agent_world_consensus/src/membership_recovery/replay_archive_federated.rs`
  - `query_revocation_dead_letter_replay_rollback_governance_audit_archive_aggregated` 的 `scanned_hot/scanned_cold` 累加从 `saturating_add` 改为受检累加。
  - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor` 的 `node_event_offset` 递进从 `saturating_add` 改为受检递进。
  - 新增受检计数 helper，溢出统一返回 `WorldError::DistributedValidationFailed`。
- 测试
  - 新增 helper overflow 单元测试，验证错误语义一致。
  - 回归现有 federated archive 查询测试，确保行为不回退。

### Out of Scope（阶段后）
- `pos.rs` / `node_pos.rs` / `sequencer_mainloop.rs` 中 epoch 推导的饱和减法语义重构。
- `membership_recovery/replay.rs` 中策略层面的饱和算术重构（非本阶段关键治理计数）。
- 全仓库数值 newtype 与静态分析 gate 统一。

## 接口/数据
- 对外 API 入口保持不变（继续返回 `Result<..., WorldError>`）。
- 内部新增受检 helper（`checked_usize_add` / `checked_usize_increment`）。
- 失败模型统一：
  - `WorldError::DistributedValidationFailed`
  - 错误消息包含 `lhs/rhs` 与计数字段上下文。

## 里程碑
- M0：Phase15 建档并冻结边界。
- M1：聚合扫描计数受检改造完成。
- M2：复合游标偏移计数受检改造与边界测试完成。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- 由饱和改为显式失败后，极端边界查询会从“夹逼继续”变为“拒绝”，需同步测试预期。
- 复合游标排序与过滤逻辑较长，改动时需避免破坏既有 cursor 语义。
- 需确保错误发生在返回结果前，不产生半截游标状态写入。

## 当前状态
- 截至 2026-02-23：M0 已完成；M1、M2、M3 待执行。
