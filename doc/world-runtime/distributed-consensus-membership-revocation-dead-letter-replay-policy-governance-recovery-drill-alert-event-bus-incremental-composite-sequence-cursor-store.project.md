# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合序号游标状态持久化（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现复合序号游标状态模型与 store 抽象
- [x] 实现内存/文件游标状态存储
- [x] 实现带状态存储的一体化增量续拉接口
- [x] 补充游标状态持久化单元测试
- [x] 完成验证并同步总项目文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive_federated.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_federated_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.41 能力实现、验证与文档同步完成）
- 下一步：P3.42 已完成，推进 P3.43（待规划）
- 最近更新：2026-02-11
