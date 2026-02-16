> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位自动推进与空拉取幂等（P3.38）

## 目标
- 在 P3.37 增量拉取基础上，补齐“查询 + 水位推进”一体化能力，降低消费端手工计算下一水位的重复逻辑。
- 明确空拉取场景的幂等语义：无新事件时保持水位不回退，便于轮询任务安全续拉。
- 保持与现有 outcome 过滤和 `max_records` 限流策略兼容。

## 范围
- **包含**：
  - 新增接口：
    - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_with_next_watermark(...)`
  - 接口返回 `(events, next_since_event_at_ms)`，其中 `next_since_event_at_ms` 自动由本轮返回结果计算。
  - 明确并实现水位单调性约束（`next_since_event_at_ms >= since_event_at_ms`）。
  - 新增单元测试覆盖分批拉取与空拉取场景。
- **不包含**：
  - 引入外部持久化 cursor store
  - 引入复合游标（时间 + 节点 + 序号）
  - 事件总线存储结构改动

## 接口/数据
- 输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`
  - `outcomes`
  - `max_records`
  - `event_bus`
- 输出：
  - `Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>`
  - `i64`（下一次增量拉取水位）
- 行为约束：
  - 仍复用既有增量逻辑：严格 `event_at_ms > since_event_at_ms`
  - 水位推进以本轮结果最后一条事件时间为准；若本轮无结果则保持原水位

## 里程碑
- M1：完成设计文档与项目管理文档。
- M2：实现增量拉取 + 下一水位联动接口。
- M3：补充分批拉取与空拉取幂等测试。
- M4：执行验证命令并同步总项目文档与 devlog。

## 风险
- **同毫秒事件边界风险**：当前仅用时间水位，仍不完全覆盖同毫秒多事件稳定续拉；后续可引入复合游标。
- **消费端误用风险**：若消费端不使用返回水位而自行计算，可能出现重复拉取或漏拉；需要在接口文档中明确推荐用法。
