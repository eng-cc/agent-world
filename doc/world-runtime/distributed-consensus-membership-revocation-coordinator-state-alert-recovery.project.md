# Agent World Runtime：成员目录吊销协同状态外部存储与告警恢复机制（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增协同状态外部存储抽象与内存/文件实现
- [x] 新增 `StoreBackedMembershipRevocationScheduleCoordinator`
- [x] 新增告警恢复存储抽象与内存/文件实现
- [x] 新增 `emit_revocation_reconcile_alerts_with_recovery(...)`
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery(...)`
- [x] 补充单元测试与回归验证

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-alert-dedup-coordination.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/reconciliation.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_tests.rs`

## 状态
- 当前阶段：MR4 完成（协同状态外部存储与告警恢复机制已落地）
- 下一步：规划恢复队列容量治理与告警 ACK 策略
- 最近更新：2026-02-11
