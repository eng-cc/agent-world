# Agent World Runtime：成员目录吊销死信回放策略治理审计归档分层转储与演练告警联动（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现治理审计归档分层转储策略与执行入口
- [x] 实现分层转储失败补偿机制
- [x] 实现恢复演练异常告警策略与告警状态存储（内存/文件）
- [x] 实现转储+演练+告警联动编排入口
- [x] 补充分层转储与演练告警联动单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_archive_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.34 能力实现、验证与文档同步完成）
- 下一步：推进 P3.35（待规划）
- 最近更新：2026-02-11
