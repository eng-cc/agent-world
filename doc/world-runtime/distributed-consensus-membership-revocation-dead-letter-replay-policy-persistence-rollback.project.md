# Agent World Runtime：成员目录吊销死信回放策略建议持久化与回滚保护（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现策略状态 store 抽象与内存/文件实现
- [x] 实现持久化推荐与回滚保护入口
- [x] 实现协同调度联动入口并补充单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_tests.rs`

## 状态
- 当前阶段：MR4 完成（策略建议持久化与回滚保护已落地）
- 下一步：P3.30/P3.31 已完成，继续推进 P3.32（治理审计归档与恢复演练）
- 最近更新：2026-02-11
