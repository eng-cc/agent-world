> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放调度与指标导出（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] 输出设计文档（`doc/p2p/distributed/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.md`） (PRD-P2P-MIG-014)
- [x] 输出项目管理文档（本文件） (PRD-P2P-MIG-014)
- [x] 扩展 dead-letter store 抽象（list/replace/metrics） (PRD-P2P-MIG-014)
- [x] 实现 `replay_revocation_dead_letters(...)` (PRD-P2P-MIG-014)
- [x] 实现 `run_revocation_dead_letter_replay_schedule(...)` (PRD-P2P-MIG-014)
- [x] 实现 metrics 导出入口与协同执行联动导出 (PRD-P2P-MIG-014)
- [x] 单元测试与分布式回归验证 (PRD-P2P-MIG-014)

## 依赖
- `doc/p2p/distributed/distributed-consensus-membership-revocation-dead-letter-metrics.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_tests.rs`

## 状态
- 当前阶段：MR4 完成（死信回放调度与指标导出已落地）
- 下一步：P3.26 已完成，后续推进 P3.27（回放状态观测聚合与策略自适应）
- 最近更新：2026-02-11
