# Agent World Runtime：成员目录吊销死信回放状态持久化与公平调度（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增 dead-letter 回放状态 store 抽象与内存/文件实现
- [x] 新增公平回放策略与策略校验
- [x] 新增基于 state store 的调度入口（含协同版本）
- [x] 补充单元测试并完成回归验证
- [x] 同步总文档与开发日志状态

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-priority-coordination.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_tests.rs`

## 状态
- 当前阶段：MR4 完成（回放状态持久化与公平调度已落地）
- 下一步：P3.27/P3.28/P3.29/P3.30/P3.31 已完成，继续推进 P3.32（治理审计归档与恢复演练）
- 最近更新：2026-02-11
