> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合序号游标状态持久化（P3.41）

## 目标
- 在 P3.40 复合序号游标查询能力基础上，补齐消费端游标状态持久化能力，支持进程重启后续拉。
- 提供“读取状态 -> 拉取增量 -> 更新状态”一体化查询入口，降低业务侧重复样板代码。
- 保持与既有 outcome 过滤、`max_records` 限流和空拉取幂等行为兼容。

## 范围
- **包含**：
  - 新增复合序号游标状态模型与 store 抽象。
  - 新增内存/文件两种状态存储实现。
  - 新增带状态存储的一体化增量续拉接口。
  - 新增单元测试覆盖：首次拉取、续拉、空拉取幂等、文件存储回读。
- **不包含**：
  - 分布式共享游标状态一致性协议
  - 基于租约/锁的多消费者排他控制
  - 对外网络协议暴露游标状态接口

## 接口/数据
- 状态模型：
  - `world_id`
  - `consumer_id`
  - `since_event_at_ms`
  - `since_node_id`
  - `since_node_event_offset`
- 新增接口（示意）：
  - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_with_composite_sequence_cursor_state(...)`
- 行为语义：
  - 若 store 无状态，使用调用方提供的初始 cursor。
  - 查询后将返回 cursor 持久化到 store。
  - 空拉取时 cursor 保持不变并持久化（幂等）。

## 里程碑
- M1：完成设计文档与项目管理文档。
- M2：实现游标状态模型与内存/文件 store。
- M3：实现带状态存储的一体化续拉接口。
- M4：补充测试、完成验证与文档/devlog 同步。

## 风险
- **并发覆盖风险**：多消费者共用同一 `consumer_id` 会互相覆盖 cursor；需业务侧明确 consumer 维度隔离。
- **文件写放大风险**：高频轮询会触发频繁落盘；后续可评估批量 flush 或节流策略。
