# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标增量续拉（P3.39）

## 目标
- 在 P3.38 时间水位推进基础上，补齐同毫秒事件场景的稳定续拉能力。
- 引入“时间水位 + 节点游标”复合 cursor，降低跨节点同时间戳事件重复消费风险。
- 保持与既有 outcome 过滤、`max_records` 限流和空拉取幂等行为兼容。

## 范围
- **包含**：
  - 新增接口：
    - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_cursor(...)`
  - 支持输入 cursor：
    - `since_event_at_ms`
    - `since_node_id`
  - 返回：
    - `events`
    - `next_event_at_ms`
    - `next_node_id`
  - 补充同毫秒多节点续拉单元测试。
- **不包含**：
  - 同节点同毫秒多事件序号级 cursor（后续里程碑）
  - 外部持久化 cursor store

## 接口/数据
- 输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`
  - `since_node_id`
  - `outcomes`
  - `max_records`
  - `event_bus`
- 过滤语义：
  - `event_at_ms > since_event_at_ms`，或
  - `event_at_ms == since_event_at_ms && node_id > since_node_id`
- 输出：
  - `(Vec<Event>, i64, Option<String>)`
  - 空结果时保持输入 cursor 不变（幂等续拉）

## 里程碑
- M1：完成设计文档与项目管理文档。
- M2：完成复合 cursor 查询实现。
- M3：完成同毫秒跨节点续拉测试。
- M4：完成验证、总项目管理文档与 devlog 更新。

## 风险
- **同节点同毫秒多事件风险**：当前 cursor 粒度到 `node_id`，对“同节点同毫秒多条”仍可能重复；后续需引入序号或事件 ID。
- **游标比较规则风险**：依赖节点 ID 字典序稳定；跨系统需统一节点 ID 归一化策略。
