# Agent World Runtime：成员目录吊销授权治理与跨节点对账（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-governance-reconcile.md`）
- [x] 输出项目管理文档（本文件）
- [x] 扩展吊销同步授权策略（`authorized_requesters`）
- [x] 实现对账 topic 与 checkpoint 消息
- [x] 实现对账策略/报告与对账入口
- [x] 补充单元测试与回归验证

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-auth-archive.md`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/logic.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/tests.rs`

## 状态
- 当前阶段：MR4 完成（授权治理与跨节点对账已落地）
- 后续进展：P3.18 已完成异常告警与对账调度自动化
- 最近更新：2026-02-10
