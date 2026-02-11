# Agent World Runtime：成员目录吊销死信回放策略治理审计归档保留策略与演练调度（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现治理审计归档保留策略与裁剪入口
- [x] 实现恢复演练调度策略与状态存储（内存/文件）
- [x] 实现演练调度编排入口并补充单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_audit.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_policy_audit_tests.rs`

## 状态
- 当前阶段：MR4 完成（归档保留与演练调度能力已落地）
- 下一步：P3.34 已完成，推进 P3.35（待规划）
- 最近更新：2026-02-11
