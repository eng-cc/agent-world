> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略采纳审计与异常回退告警（设计文档）

## 目标
- 为 dead-letter 回放策略推荐链路补齐“采纳审计”记录，确保每次策略采用/回滚都可追踪。
- 在策略回滚异常集中出现时触发告警，形成回滚保护后的异常闭环。
- 保持现有持久化推荐与协同调度入口兼容，新增“审计+告警”增强入口。

## 范围

### In Scope（本次实现）
- 新增策略采纳审计模型与审计存储抽象（内存/文件实现）。
- 新增回滚异常告警策略与告警状态模型，支持窗口计数与告警冷却。
- 新增联动入口：持久化推荐 + 回滚保护 + 采纳审计 + 异常回退告警。
- 补充单元测试：审计存储 round-trip、审计落盘、回滚告警触发与冷却抑制。

### Out of Scope（本次不做）
- 跨节点统一审计聚合与集中查询服务。
- 告警分级路由到外部告警平台（PagerDuty/IM）适配。
- 基于历史审计自动调整 rollback guard 阈值。

## 接口 / 数据

### 策略采纳审计记录（拟）
- `MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord`
  - `world_id/node_id/audited_at_ms`
  - `decision`：`Adopted` / `RolledBackToStable`
  - `recommended_policy/applied_policy/stable_policy`
  - `backlog_dead_letters/backlog_pending`
  - `metrics`（attempted/failed/dead_lettered 等聚合值）
  - `rollback_triggered`

### 审计存储（拟）
- `MembershipRevocationDeadLetterReplayPolicyAuditStore`
  - `append(record)`
  - `list(world_id, node_id)`
- 实现：`InMemory...AuditStore`、`File...AuditStore`（JSONL）。

### 回滚异常告警策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackAlertPolicy`
  - `rollback_window_ms`：统计窗口
  - `max_rollbacks_per_window`：窗口阈值
  - `min_attempted`：触发告警所需最小投递样本
  - `alert_cooldown_ms`：告警冷却
- `MembershipRevocationDeadLetterReplayRollbackAlertState`
  - `last_alert_at_ms`

### 联动入口（拟）
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_and_alert(...)`
  - 输出：`(replayed, policy, rolled_back, alert_emitted)`。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：采纳审计模型、存储与回滚异常告警策略实现完成。
- **MR3**：联动执行入口与测试完成。
- **MR4**：总文档、项目状态、开发日志同步完成。

## 风险
- 告警阈值过敏会造成高频噪声，需依赖冷却窗口约束。
- 审计记录长期累积可能带来文件增长，需要后续归档策略。
- 告警字段复用现有异常模型时语义可能偏弱，需要后续统一告警 schema。
