> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销来源鉴权与审计落盘归档（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/p2p/distributed-consensus-membership-revocation-auth-archive.md`）
- [x] 输出项目管理文档（本文件）
- [x] 扩展吊销消息结构（signature_key_id/signature）
- [x] 实现吊销消息签名/验签能力（signer + keyring）
- [x] 实现吊销同步策略与校验流程（policy + report）
- [x] 实现落盘审计存储 `FileMembershipAuditStore`
- [x] 补充单元测试与回归验证

## 依赖
- `doc/p2p/distributed-consensus-membership-audit-revocation.md`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/tests.rs`

## 状态
- 当前阶段：MR4 完成（吊销来源鉴权与审计落盘已落地）
- 后续进展：P3.17 已完成吊销授权治理与跨节点对账
- 最近更新：2026-02-10
