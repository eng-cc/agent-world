> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放调度与指标导出（设计文档）

## 目标
- 为成员目录吊销告警死信队列提供可调度回放能力，将可恢复死信重新注入恢复队列。
- 为告警投递指标提供统一导出能力（内存/文件），便于后续对接监控与离线分析。
- 保持现有 ACK 重试与死信归档链路兼容，支持增量启用。

## 范围

### In Scope（本次实现）
- 扩展 `MembershipRevocationAlertDeadLetterStore`：支持 list/replace 与 delivery metrics append/list。
- 补齐内存/文件 dead-letter store 对应实现。
- 新增 `replay_revocation_dead_letters(...)`：按上限批量回放 dead-letter 到 recovery pending 队列。
- 新增 `run_revocation_dead_letter_replay_schedule(...)`：按间隔调度 dead-letter 回放。
- 新增 `export_revocation_alert_delivery_metrics(...)` 与协同调度联动导出入口。
- 补充单元测试覆盖回放顺序、调度触发与指标导出。

### Out of Scope（本次不做）
- 基于告警等级/错误码的差异化回放策略。
- 死信回放去重与跨节点回放协调锁。
- 对接外部监控协议（Prometheus/OpenTelemetry）。

## 接口 / 数据

### Dead-letter store 扩展
- `list(world_id, node_id)`
- `replace(world_id, node_id, records)`
- `append_delivery_metrics(world_id, node_id, exported_at_ms, metrics)`
- `list_delivery_metrics(world_id, node_id)`

### 回放入口
- `replay_revocation_dead_letters(...)`
  - 从 dead-letter store 拉取记录
  - 取前 N 条回放到 pending 队列
  - 余量回写 dead-letter store

### 调度入口
- `run_revocation_dead_letter_replay_schedule(...)`
  - 基于 `replay_interval_ms` 判定是否到期
  - 到期时执行 replay 并更新 `last_replay_at_ms`

### 指标导出
- `export_revocation_alert_delivery_metrics(...)`
  - 将 `MembershipRevocationAlertDeliveryMetrics` 导出到 store
- `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter_and_metrics_export(...)`
  - 协同执行后自动导出本次 delivery metrics

## 里程碑
- **MR1**：设计文档与项目文档完成。
- **MR2**：dead-letter store 扩展与实现完成。
- **MR3**：回放调度与指标导出入口完成。
- **MR4**：测试、总文档、项目状态同步完成。

## 风险
- 回放策略当前按 FIFO + fixed limit，缺少优先级控制。
- 指标导出与主流程共享存储时，写入失败可能影响主流程可观测性。
- 大规模回放可能与在线新告警竞争 recovery 队列容量。
