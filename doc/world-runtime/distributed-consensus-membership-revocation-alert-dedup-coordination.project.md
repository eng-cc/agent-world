# Agent World Runtime：成员目录吊销告警抑制去重与调度多节点协同（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-alert-dedup-coordination.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增告警去重策略/状态与去重入口
- [x] 新增调度协同抽象与内存实现
- [x] 新增协同调度编排入口
- [x] 补充单元测试与回归验证

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-alert-delivery-state-store.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/reconciliation.rs`

## 状态
- 当前阶段：MR4 完成（去重抑制与协同调度已落地）
- 下一步：规划协调器持久化与告警恢复策略
- 最近更新：2026-02-10
