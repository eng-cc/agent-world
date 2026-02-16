> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计归档与恢复演练（设计文档）

## 目标
- 为回退治理链路补齐治理级别审计归档，形成可追溯的治理历史。
- 提供恢复演练入口，验证重启后告警状态、治理状态与治理审计是否可恢复。
- 保持现有审计/告警/治理入口兼容，新增“归档+演练”增强入口。

## 范围

### In Scope（本次实现）
- 新增治理审计归档模型与归档存储抽象（内存/文件实现）。
- 新增治理联动执行增强入口：治理执行后自动归档治理审计记录。
- 新增恢复演练入口：读取 alert state、governance state 与治理审计历史并输出报告。
- 补充单元测试：归档落盘 round-trip、归档联动、恢复演练报告与参数校验。

### Out of Scope（本次不做）
- 外部审计平台（ELK/ClickHouse）适配。
- 治理演练的自动定时触发与任务调度。
- 跨节点统一治理审计聚合查询。

## 接口 / 数据

### 治理审计归档记录（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord`
  - `world_id/node_id/audited_at_ms`
  - `governance_level`（`normal/stable/emergency`）
  - `rollback_streak`
  - `rolled_back`
  - `applied_policy`
  - `alert_emitted`

### 治理审计归档 store（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore`
  - `append(world_id, node_id, record)`
  - `list(world_id, node_id)`
- 实现：`InMemory...AuditStore`、`File...AuditStore`（JSONL）。

### 恢复演练报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport`
  - `world_id/node_id/drill_at_ms`
  - `alert_state`
  - `governance_state`
  - `recent_audits`
  - `has_emergency_history`

### 联动入口（拟）
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_governance_and_archive(...)`
- `run_revocation_dead_letter_replay_rollback_governance_recovery_drill(...)`

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：治理审计归档模型与存储实现完成。
- **MR3**：归档联动入口与恢复演练入口、测试完成。
- **MR4**：总文档、项目状态、开发日志同步完成。

## 风险
- 归档记录增长较快，后续需补充裁剪或分层归档策略。
- 恢复演练报告字段过于简化时，可能不足以支撑复杂故障复盘。
- 演练入口与真实恢复流程偏离时可能产生误判，需要保持语义对齐。
