# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线聚合拉取与分页增量查询（P3.36）

## 目标
- 在 P3.35 事件总线写入能力基础上，补齐读侧能力，支持跨节点聚合拉取恢复演练告警事件。
- 提供统一的时间窗过滤、outcome 过滤与分页增量查询能力，降低上层治理面板/巡检任务的重复聚合成本。
- 保持现有事件总线实现（内存/文件）兼容，不引入破坏性接口变更。

## 范围
- **包含**：
  - 新增 `MembershipSyncClient` 事件总线聚合查询入口：
    - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated(...)`
  - 支持过滤维度：
    - `min_event_at_ms`（时间下界）
    - `outcomes`（按 `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome` 过滤）
  - 支持分页增量：
    - `offset` + `max_records`
  - 新增对应单元测试（聚合过滤、排序分页、参数校验）。
- **不包含**：
  - 事件总线存储格式变更
  - 新增外部协议/API（HTTP/gRPC）
  - 消费游标持久化（后续里程碑再补）

## 接口/数据
- 查询输入：
  - `world_id`
  - `node_ids`
  - `min_event_at_ms`
  - `outcomes`
  - `offset`
  - `max_records`
  - `event_bus`
- 查询输出：
  - `Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>`
- 行为约束：
  - `node_ids` 不能为空；
  - `max_records` 必须大于 0；
  - 聚合结果按 `event_at_ms desc` + `node_id asc` 稳定排序；
  - 聚合时仅保留与查询维度一致的 `world_id/node_id` 事件记录。

## 里程碑
- M1：完成设计文档与项目管理文档。
- M2：完成 `replay_archive_federated.rs` 聚合拉取实现与参数校验。
- M3：完成 `recovery_replay_federated_tests.rs` 单元测试覆盖（过滤、分页、异常参数）。
- M4：运行格式化、定向测试与模块回归，更新总项目管理文档与 devlog。

## 风险
- **事件量增长风险**：当前文件事件总线 `list` 读全量后过滤，超大规模下会有内存与延迟压力；后续可补按范围读取接口。
- **过滤歧义风险**：若写入端出现错误 `world_id/node_id`，读侧需显式过滤并保守忽略异常事件，避免跨节点污染。
- **分页一致性风险**：基于偏移分页在并发写入场景可能出现“翻页漂移”；后续可引入游标（事件时间+序号）改善一致性。
