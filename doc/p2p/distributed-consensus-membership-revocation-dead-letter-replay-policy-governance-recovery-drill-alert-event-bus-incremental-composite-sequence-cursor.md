# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标序号续拉（P3.40）

## 目标
- 在 P3.39 “时间水位 + 节点游标”基础上，补齐同节点同毫秒多事件的稳定续拉能力。
- 增加序号级游标维度，避免 `max_records` 分页时出现“同节点同毫秒后续事件漏拉”。
- 保持与既有 outcome 过滤、`max_records` 限流和空拉取幂等行为兼容。

## 范围
- **包含**：
  - 新增接口：
    - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor(...)`
  - 支持输入 cursor：
    - `since_event_at_ms`
    - `since_node_id`
    - `since_node_event_offset`
  - 返回：
    - `events`
    - `next_event_at_ms`
    - `next_node_id`
    - `next_node_event_offset`
  - 增补同节点同毫秒多事件的分页续拉单元测试。
- **不包含**：
  - 外部持久化 cursor store
  - 对事件总线存储格式新增事件唯一 ID 字段
  - 历史聚合查询接口签名改造

## 接口/数据
- 输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`
  - `since_node_id`
  - `since_node_event_offset`
  - `outcomes`
  - `max_records`
  - `event_bus`
- 排序与过滤语义（复合键）：
  - `(event_at_ms, node_id, node_event_offset)` 严格大于 cursor 才返回
  - 其中 `node_event_offset` 为“同一 node_id 内原始事件列表顺序索引”
- 输出：
  - `(Vec<Event>, i64, Option<String>, usize)`
  - 空结果时保持输入 cursor 不变（幂等续拉）

## 里程碑
- M1：完成设计文档与项目管理文档。
- M2：完成序号级复合游标查询实现。
- M3：完成同节点同毫秒分页续拉测试。
- M4：完成验证、总项目管理文档与 devlog 更新。

## 风险
- **顺序语义风险**：`node_event_offset` 依赖事件总线 list 顺序稳定；若底层实现改为无序存储，会破坏 cursor 一致性。
- **跨实现一致性风险**：内存与文件事件总线需保证 list 顺序语义一致（当前均为 append 顺序）。
