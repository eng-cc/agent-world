# Agent World Runtime：成员目录吊销恢复队列容量治理与告警 ACK 重试（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/p2p/distributed-consensus-membership-revocation-recovery-queue-ack-retry.md`）
- [x] 输出项目管理文档（本文件）
- [x] 定义恢复队列元素结构与 ACK 重试策略
- [x] 升级 recovery store（内存/文件）持久化格式并兼容旧格式
- [x] 新增 `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry(...)`
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry(...)`
- [x] 扩展恢复报告字段并补充单元测试

## 依赖
- `doc/p2p/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_tests.rs`

## 状态
- 当前阶段：MR4 完成（恢复队列容量治理与 ACK 重试已落地）
- 下一步：已衔接 P3.23（死信归档与投递指标），后续可推进 P3.24（死信回放调度与指标导出）
- 最近更新：2026-02-11
