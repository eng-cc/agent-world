# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线聚合拉取与分页增量查询（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现事件总线跨节点聚合拉取入口
- [x] 实现最小时间窗口 + outcome 过滤
- [x] 实现 offset + max_records 分页增量查询
- [x] 补充聚合拉取/分页/参数校验单元测试
- [x] 完成验证并同步总项目文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive_federated.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_federated_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.36 能力实现、验证与文档同步完成）
- 下一步：推进 P3.37（待规划）
- 最近更新：2026-02-11
