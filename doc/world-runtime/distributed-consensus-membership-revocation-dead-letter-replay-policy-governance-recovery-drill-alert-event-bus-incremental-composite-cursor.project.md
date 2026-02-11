# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标增量续拉（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-cursor.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现时间水位 + 节点游标复合续拉接口
- [x] 实现同毫秒跨节点事件续拉规则
- [x] 补充复合游标续拉单元测试
- [x] 完成验证并同步总项目文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive_federated.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_federated_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.39 能力实现、验证与文档同步完成）
- 下一步：P3.40 已完成，推进 P3.41（待规划）
- 最近更新：2026-02-11
