# Agent World Runtime：成员目录吊销告警上报与调度状态持久化（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-alert-delivery-state-store.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增 `MembershipRevocationAlertSink` 抽象与内存/文件实现
- [x] 新增 `MembershipRevocationScheduleStateStore` 抽象与内存/文件实现
- [x] 新增 `emit_revocation_reconcile_alerts(...)` 告警上报入口
- [x] 新增 `run_revocation_reconcile_schedule_with_store_and_alerts(...)` 编排入口
- [x] 补充单元测试与回归验证

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-alerting-scheduler.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/reconciliation.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/persistence_tests.rs`

## 状态
- 当前阶段：MR4 完成（告警上报与调度状态持久化已落地）
- 下一步：规划告警去重与多节点调度状态协同
- 最近更新：2026-02-10
