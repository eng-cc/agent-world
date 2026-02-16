# Agent World Runtime：成员目录吊销死信回放策略治理审计归档与恢复演练（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/p2p/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现治理审计归档模型与归档 store（内存/文件）
- [x] 实现归档联动执行入口与恢复演练入口
- [x] 补充归档与恢复演练单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/p2p/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_audit.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_policy_audit_tests.rs`

## 状态
- 当前阶段：MR4 完成（治理审计归档与恢复演练能力已落地）
- 下一步：P3.33/P3.34/P3.35 已完成，推进 P3.36（待规划）
- 最近更新：2026-02-11
