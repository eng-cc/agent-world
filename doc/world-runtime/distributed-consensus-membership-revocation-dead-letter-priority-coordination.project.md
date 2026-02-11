# Agent World Runtime：成员目录吊销死信优先级回放与跨节点回放协同（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-priority-coordination.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现 dead-letter 优先级回放规则（reason/attempt/dropped_at 排序）
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated(...)`
- [x] 补充优先级与协同单元测试
- [x] 执行回归验证并同步总文档/开发日志状态

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_tests.rs`

## 状态
- 当前阶段：MR4 完成（优先级回放与跨节点回放协同已落地）
- 下一步：规划 P3.26（死信回放状态持久化与公平调度）
- 最近更新：2026-02-11
