# Agent World Runtime：成员目录吊销死信回放策略治理审计聚合查询与演练告警事件总线（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现治理审计 hot/cold 聚合查询策略与查询入口
- [x] 实现聚合查询报告与参数校验
- [x] 实现恢复演练告警事件模型与事件总线（内存/文件）
- [x] 实现转储+演练+告警+事件总线联动入口
- [x] 补充聚合查询与事件总线联动单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive_tiered.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_archive_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.35 能力实现、验证与文档同步完成）
- 下一步：推进 P3.36（待规划）
- 最近更新：2026-02-11
