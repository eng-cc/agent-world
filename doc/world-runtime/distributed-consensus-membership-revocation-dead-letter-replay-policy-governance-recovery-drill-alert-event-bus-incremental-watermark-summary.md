# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位拉取与 outcome 汇总（P3.37）

## 目标
- 在 P3.36 分页聚合查询基础上，补齐“增量消费友好”的查询能力，支持按时间水位做严格增量拉取。
- 提供 outcome 维度的聚合汇总，便于巡检任务与监控面板快速判断告警结构变化。
- 保持事件总线读写接口兼容，不引入存储格式变更。

## 范围
- **包含**：
  - 新增增量拉取入口：
    - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since(...)`
  - 新增 outcome 聚合入口：
    - `summarize_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated_by_outcome(...)`
  - 复用并抽取聚合查询公共读取逻辑，统一 `world/node` 过滤一致性。
  - 补充增量水位与 outcome 汇总单元测试。
- **不包含**：
  - 消费游标状态持久化
  - 新增外部 API 层协议
  - 事件总线写入语义变更

## 接口/数据
- 增量拉取输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`（严格 `>`）
  - `outcomes`（可选过滤）
  - `max_records`
- 增量拉取输出：
  - `Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>`
  - 结果按 `event_at_ms asc + node_id asc` 排序，便于消费端顺序处理。
- outcome 汇总输出：
  - `BTreeMap<String, usize>`，key 使用稳定标签：
    - `emitted`
    - `suppressed_cooldown`
    - `suppressed_no_anomaly`
    - `skipped_no_drill`

## 里程碑
- M1：完成设计文档与项目管理文档。
- M2：完成 federated 查询实现（增量水位拉取 + outcome 汇总）。
- M3：完成单元测试覆盖（严格水位、过滤、limit、汇总）。
- M4：完成验证、项目总文档与 devlog 更新。

## 风险
- **同毫秒并发事件风险**：纯时间水位无法完全区分同毫秒事件顺序；后续可引入复合游标（时间+节点+序号）。
- **读放大风险**：当前读取路径仍基于事件总线 `list` 后过滤，极端高基数场景仍有扫描开销。
- **汇总标签扩展风险**：若新增 outcome 枚举，需要同步更新标签映射与统计逻辑，避免遗漏分类。
