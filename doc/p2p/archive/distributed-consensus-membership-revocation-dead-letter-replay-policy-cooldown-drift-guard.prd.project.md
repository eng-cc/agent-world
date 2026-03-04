> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略冷却窗口与漂移抑制（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] 输出设计文档（`doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.prd.md`） (PRD-P2P-MIG-018)
- [x] 输出项目管理文档（本文件） (PRD-P2P-MIG-018)
- [x] 实现带 guard 的策略推荐入口 (PRD-P2P-MIG-018)
- [x] 实现带 guard 的协调调度入口 (PRD-P2P-MIG-018)
- [x] 补充冷却窗口与漂移抑制单元测试 (PRD-P2P-MIG-018)
- [x] 完成验证并同步总文档/devlog (PRD-P2P-MIG-018)

## 依赖
- `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.prd.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_tests.rs`

## 状态
- 当前阶段：MR4 完成（策略冷却窗口与漂移抑制已落地）
- 下一步：P3.29/P3.30/P3.31 已完成，继续推进 P3.32（治理审计归档与恢复演练）
- 最近更新：2026-02-11
