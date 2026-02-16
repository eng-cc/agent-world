> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销告警恢复死信归档与投递指标（设计文档）

## 目标
- 为成员目录吊销告警恢复链路补齐死信归档机制，避免重试耗尽/容量裁剪时告警静默丢失。
- 为告警投递过程提供结构化指标（attempt/success/failure/deferred/drop），提升恢复链路可观测性。
- 与现有恢复队列、ACK 重试、协同调度接口保持兼容，支持增量接入。

## 范围

### In Scope（本次实现）
- 新增死信归档模型与存储抽象：
  - `MembershipRevocationAlertDeadLetterReason`
  - `MembershipRevocationAlertDeadLetterRecord`
  - `MembershipRevocationAlertDeadLetterStore`
  - `InMemoryMembershipRevocationAlertDeadLetterStore`
  - `FileMembershipRevocationAlertDeadLetterStore`
- 新增投递指标结构：
  - `MembershipRevocationAlertDeliveryMetrics`
- 扩展恢复执行报告：
  - `MembershipRevocationAlertRecoveryReport` 增加 `delivery_metrics`
  - `MembershipRevocationCoordinatedRecoveryRunReport` 增加 `delivery_metrics`
- 新增支持死信归档的执行入口：
  - `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter(...)`
  - `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter(...)`
- 兼容旧入口：原有方法通过 noop dead-letter store 继续可用。

### Out of Scope（本次不做）
- 死信再投递调度器（自动 replay DLQ）。
- 死信内容脱敏/加密策略与审计访问控制。
- Prometheus/OpenTelemetry exporter 适配。

## 接口 / 数据

### 死信记录
- `MembershipRevocationAlertDeadLetterRecord`
  - `world_id`
  - `node_id`
  - `dropped_at_ms`
  - `reason`
  - `pending_alert`

### 死信原因
- `MembershipRevocationAlertDeadLetterReason`
  - `retry_limit_exceeded`
  - `capacity_evicted`

### 投递指标
- `MembershipRevocationAlertDeliveryMetrics`
  - `attempted`
  - `succeeded`
  - `failed`
  - `deferred`
  - `buffered`
  - `dropped_capacity`
  - `dropped_retry_limit`
  - `dead_lettered`

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：死信归档模型/存储抽象与实现完成。
- **MR3**：恢复执行入口接入死信归档并补齐投递指标。
- **MR4**：协同执行入口接入、测试完善、总文档与状态同步完成。

## 风险
- 死信归档在极端故障下可能快速增长，需要后续引入 TTL 或归档压缩。
- 文件型死信归档在多进程并发 append 下仍有竞争风险。
- 指标为流程内聚合值，未接入外部监控时仍需主动拉取/日志采集。
