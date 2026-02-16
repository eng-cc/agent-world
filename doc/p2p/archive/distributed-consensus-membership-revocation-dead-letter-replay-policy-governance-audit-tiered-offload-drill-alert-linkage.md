> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计归档分层转储与演练告警联动（设计文档）

## 目标
- 为治理审计归档增加热/冷分层能力，避免热归档无限增长并保留近期可快速查询窗口。
- 提供可重入的分层转储入口，在失败场景下提供补偿回滚，避免冷热层数据失配。
- 在恢复演练调度链路增加异常告警联动，实现“演练执行→异常判定→告警落盘”闭环。

## 范围

### In Scope（本次实现）
- 新增治理审计归档分层转储策略模型（热层保留、转储年龄门槛、单次转储上限）。
- 新增分层转储执行入口与运行报告，支持热/冷归档存储协同。
- 新增转储失败补偿机制（热层写失败时回滚冷层写入）。
- 新增恢复演练异常告警策略、告警状态存储（内存/文件）与告警执行入口。
- 新增“归档保留+分层转储+演练调度+告警联动”编排入口。
- 补充单元测试：分层转储、补偿回滚、告警冷却、联动编排。

### Out of Scope（本次不做）
- 冷层对象存储（S3/OSS）适配与远程生命周期管理。
- 多节点统一冷热层聚合查询。
- 告警通知外送（Webhook/IM）与事件总线。

## 接口 / 数据

### 分层转储策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadPolicy`
  - `hot_max_records`
  - `offload_min_age_ms`
  - `max_offload_records`

### 分层转储报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadReport`
  - `world_id/node_id/offloaded_at_ms`
  - `hot_before/hot_after`
  - `cold_before/cold_after`
  - `offloaded`
  - `offloaded_by_age/offloaded_by_capacity`
  - `kept_due_to_rate_limit`

### 演练告警策略与状态（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertPolicy`
  - `max_alert_silence_ms`
  - `rollback_streak_threshold`
  - `alert_cooldown_ms`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertState`
  - `last_alert_at_ms`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore`
  - `load/save`（内存/文件实现）

### 联动编排报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertRunReport`
  - `prune_report`
  - `offload_report`
  - `drill_run_report`
  - `drill_alert_report`

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：分层转储策略、执行入口与补偿机制实现完成。
- **MR3**：演练异常告警策略/状态存储/告警入口实现完成。
- **MR4**：联动编排入口、测试、总文档与 devlog 同步完成。

## 风险
- 分层阈值设置不当可能导致热层抖动或冷层增长过快。
- 补偿回滚仅覆盖冷热层写顺序失配，仍需后续补齐跨进程幂等保障。
- 告警判定规则过严会产生噪声，过松会漏报，需要后续基于线上指标调优。
