> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计归档保留策略与演练调度（设计文档）

## 目标
- 为治理审计归档增加可配置保留策略，控制历史记录增长并保留关键窗口数据。
- 提供可重复执行的归档裁剪入口，支持按条数与时间窗口双维治理。
- 为恢复演练新增调度编排能力，支持周期化演练与状态持久化。

## 范围

### In Scope（本次实现）
- 新增治理审计归档保留策略模型（按最大记录数、最大保留时长）。
- 新增治理审计归档裁剪入口与执行报告。
- 新增恢复演练调度策略、调度状态存储（内存/文件）与调度编排入口。
- 补充单元测试：裁剪策略、文件裁剪、调度间隔门控、参数校验。

### Out of Scope（本次不做）
- 多 world 的统一归档策略中心化管理。
- 审计记录冷热分层、对象存储下沉。
- 外部任务系统（cron/k8s job）联动。

## 接口 / 数据

### 归档保留策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy`
  - `max_records: usize`（最多保留记录数）
  - `max_age_ms: i64`（最大保留时长，基于 `audited_at_ms`）

### 归档裁剪报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditPruneReport`
  - `world_id/node_id/pruned_at_ms`
  - `before/after`
  - `pruned_by_age/pruned_by_capacity`

### 演练调度策略与状态（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy`
  - `drill_interval_ms`
  - `recent_audit_limit`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleState`
  - `last_drill_at_ms`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore`
  - `load/save`（内存/文件实现）

### 调度运行报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduledRunReport`
  - `drill_due`
  - `drill_executed`
  - `drill_report`（可选，复用 drill 报告）

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：归档保留策略与裁剪入口实现完成。
- **MR3**：恢复演练调度与状态存储实现完成。
- **MR4**：测试、总文档、项目状态、devlog 同步完成。

## 风险
- 裁剪策略过于激进时，可能影响事后审计深度。
- 时间窗口依赖 `now_ms`，时钟漂移可能影响裁剪与调度边界。
- 调度状态与业务状态更新时序不一致时，可能出现重复演练或漏演练。
