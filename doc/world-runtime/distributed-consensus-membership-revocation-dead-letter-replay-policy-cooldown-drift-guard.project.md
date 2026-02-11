# Agent World Runtime：成员目录吊销死信回放策略冷却窗口与漂移抑制（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现带 guard 的策略推荐入口
- [x] 实现带 guard 的协调调度入口
- [x] 补充冷却窗口与漂移抑制单元测试
- [x] 完成验证并同步总文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_tests.rs`

## 状态
- 当前阶段：MR4 完成（策略冷却窗口与漂移抑制已落地）
- 下一步：P3.29/P3.30 已完成，继续推进 P3.31（策略审计状态持久化与多级回退治理）
- 最近更新：2026-02-11
