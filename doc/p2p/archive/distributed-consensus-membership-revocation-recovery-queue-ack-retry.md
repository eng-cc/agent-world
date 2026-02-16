> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销恢复队列容量治理与告警 ACK 重试（设计文档）

## 目标
- 为成员目录吊销告警恢复队列引入容量治理，避免下游长期不可用导致无界积压。
- 为告警恢复链路引入 ACK 重试策略（重试次数与重试退避），提高短时故障下的投递成功率。
- 在保持现有恢复/协同接口兼容的前提下，增加可观测报告字段，支撑运维诊断。

## 范围

### In Scope（本次实现）
- 新增恢复队列元素结构，持久化 `attempt`、`next_retry_at_ms` 等 ACK 重试元数据。
- 新增 ACK 重试策略：
  - `max_pending_alerts`
  - `max_retry_attempts`
  - `retry_backoff_ms`
- 新增带 ACK 重试的发送入口：
  - `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry(...)`
- 新增带 ACK 重试的协同编排入口：
  - `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry(...)`
- 扩展恢复报告，增加容量丢弃、重试丢弃、延后重试等统计字段。
- 补充单元测试覆盖容量治理、退避重试、最大重试丢弃、协同编排集成路径。

### Out of Scope（本次不做）
- 多级退避（指数退避/jitter）与按告警等级差异化策略。
- 告警投递死信队列（DLQ）归档与专用检索接口。
- 外部消息系统（Kafka/NATS）投递 ACK 协议适配。

## 接口 / 数据

### 恢复队列元素
- `MembershipRevocationPendingAlert`
  - `alert: MembershipRevocationAnomalyAlert`
  - `attempt: usize`
  - `next_retry_at_ms: i64`
  - `last_error: Option<String>`

### ACK 重试策略
- `MembershipRevocationAlertAckRetryPolicy`
  - `max_pending_alerts`: 队列最大容量，超限触发容量治理。
  - `max_retry_attempts`: 单告警最大 ACK 失败重试次数。
  - `retry_backoff_ms`: ACK 失败后的最短重试等待时间。

### 恢复发送与协同编排
- `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry(...)`
  - 读取 pending 队列。
  - 对到期条目执行 ACK 投递；失败则更新 attempt/next_retry。
  - 新告警尝试即时发送；失败入队。
  - 按 `max_pending_alerts` 执行容量裁剪并回写。
- `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry(...)`
  - 与既有协调器 + schedule + dedup 流程集成。
  - 在告警发射阶段使用 ACK 重试策略。

### 报告字段扩展
- `MembershipRevocationAlertRecoveryReport`
  - `recovered`
  - `emitted_new`
  - `buffered`
  - `deferred`
  - `dropped_capacity`
  - `dropped_retry_limit`

## 里程碑
- **MR1**：设计/项目文档完成。
- **MR2**：恢复队列结构与存储格式升级完成（含兼容读取）。
- **MR3**：ACK 重试 + 容量治理发送入口完成。
- **MR4**：协同编排入口集成、单元测试与总文档更新完成。

## 风险
- 队列容量治理策略若配置过小，可能在持续故障时丢弃过多新告警。
- 统一退避参数在异构下游通道下可能不够精细，存在恢复速度与压力折中。
- 旧格式恢复文件兼容读取若异常，需要明确回退为安全空队列并记录错误。
