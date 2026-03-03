> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销告警恢复死信归档与投递指标（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] 输出设计文档（`doc/p2p/distributed/distributed-consensus-membership-revocation-dead-letter-metrics.md`） (PRD-P2P-MIG-012)
- [x] 输出项目管理文档（本文件） (PRD-P2P-MIG-012)
- [x] 新增死信归档模型与存储抽象（内存/文件） (PRD-P2P-MIG-012)
- [x] 扩展恢复报告投递指标结构 (PRD-P2P-MIG-012)
- [x] 新增 `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter(...)` (PRD-P2P-MIG-012)
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter(...)` (PRD-P2P-MIG-012)
- [x] 单元测试与分布式回归验证 (PRD-P2P-MIG-012)

## 依赖
- `doc/p2p/distributed/distributed-consensus-membership-revocation-recovery-queue-ack-retry.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_tests.rs`

## 状态
- 当前阶段：MR4 完成（死信归档与投递指标已落地）
- 下一步：已衔接 P3.24（死信回放调度与指标导出），后续可推进 P3.25（死信优先级回放与跨节点回放协同）
- 最近更新：2026-02-11
