# Agent World Runtime：成员目录吊销死信回放状态观测聚合与策略自适应（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现回放观测聚合与策略推荐入口
- [x] 实现推荐后执行的协调调度入口
- [x] 补充策略自适应单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_tests.rs`

## 状态
- 当前阶段：MR4 完成（观测聚合与策略自适应已落地）
- 下一步：P3.28 已完成，继续推进 P3.29（策略建议持久化与回滚保护）
- 最近更新：2026-02-11
