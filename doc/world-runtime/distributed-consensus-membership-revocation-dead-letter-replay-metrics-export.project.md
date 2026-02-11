# Agent World Runtime：成员目录吊销死信回放调度与指标导出（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.md`）
- [x] 输出项目管理文档（本文件）
- [x] 扩展 dead-letter store 抽象（list/replace/metrics）
- [x] 实现 `replay_revocation_dead_letters(...)`
- [x] 实现 `run_revocation_dead_letter_replay_schedule(...)`
- [x] 实现 metrics 导出入口与协同执行联动导出
- [x] 单元测试与分布式回归验证

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-metrics.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_tests.rs`

## 状态
- 当前阶段：MR4 完成（死信回放调度与指标导出已落地）
- 下一步：规划死信优先级回放与跨节点回放协同（P3.25）
- 最近更新：2026-02-11
