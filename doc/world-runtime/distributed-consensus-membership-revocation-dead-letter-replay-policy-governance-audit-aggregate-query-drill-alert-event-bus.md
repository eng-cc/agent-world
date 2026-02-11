# Agent World Runtime：成员目录吊销死信回放策略治理审计聚合查询与演练告警事件总线（设计文档）

## 目标
- 补齐治理审计冷热归档的跨节点聚合查询能力，支持故障复盘时一次性查看多节点历史。
- 将恢复演练告警结果标准化为事件并写入事件总线，形成“判定→告警→事件追踪”闭环。
- 在保持现有归档分层与演练告警流程兼容的前提下，新增可选事件总线联动入口。

## 范围

### In Scope（本次实现）
- 新增治理审计归档聚合查询策略与查询报告（支持 hot/cold 分层、时间窗、级别过滤、条数限制）。
- 新增跨节点聚合查询入口，支持按 node 列表聚合并按时间排序返回。
- 新增恢复演练告警事件模型与事件总线抽象（内存/文件实现）。
- 新增“归档分层+演练告警+事件总线”联动入口，执行后同步落盘事件。
- 补充单元测试：聚合过滤/排序、事件总线落盘 round-trip、联动入口事件写入。

### Out of Scope（本次不做）
- 冷层对象存储（S3/OSS）适配与远端生命周期治理。
- 跨进程消息队列（Kafka/NATS）投递与 ACK 重试语义。
- 对外通知适配器（Webhook/IM）细分路由策略。

## 接口 / 数据

### 聚合查询策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy`
  - `include_hot`
  - `include_cold`
  - `max_records`
  - `min_audited_at_ms`
  - `levels`

### 聚合查询结果（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord`
  - `world_id/node_id`
  - `tier`（`hot/cold`）
  - `audit`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport`
  - `world_id/queried_node_count`
  - `scanned_hot/scanned_cold`
  - `returned`
  - `records`

### 演练告警事件总线（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent`
  - `world_id/node_id/event_at_ms`
  - `outcome`（`emitted/suppressed_cooldown/suppressed_no_anomaly/skipped_no_drill`）
  - `reasons`
  - `severity`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus`
  - `publish(...)`
  - `list(world_id, node_id)`
- 实现：`InMemory...EventBus`、`File...EventBus`（JSONL）。

### 联动入口（拟）
- `run_revocation_dead_letter_replay_rollback_governance_archive_tiered_offload_with_drill_schedule_alert_and_event_bus(...)`

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：聚合查询策略、报告与查询入口实现完成。
- **MR3**：演练告警事件总线（内存/文件）与事件映射实现完成。
- **MR4**：联动入口、测试、总文档与 devlog 同步完成。

## 风险
- 聚合查询在节点数较多时会增加扫描成本，后续可能需要分页与索引。
- hot/cold 同时查询时可能出现重复语义，需要明确 tier 与去重策略。
- 事件总线先落本地文件，跨进程消费一致性需后续阶段补齐。
