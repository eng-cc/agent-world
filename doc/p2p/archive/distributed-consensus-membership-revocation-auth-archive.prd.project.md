> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销来源鉴权与审计落盘归档（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] 输出设计文档（`doc/p2p/archive/distributed-consensus-membership-revocation-auth-archive.prd.md`） (PRD-P2P-MIG-010)
- [x] 输出项目管理文档（本文件） (PRD-P2P-MIG-010)
- [x] 扩展吊销消息结构（signature_key_id/signature） (PRD-P2P-MIG-010)
- [x] 实现吊销消息签名/验签能力（signer + keyring） (PRD-P2P-MIG-010)
- [x] 实现吊销同步策略与校验流程（policy + report） (PRD-P2P-MIG-010)
- [x] 实现落盘审计存储 `FileMembershipAuditStore` (PRD-P2P-MIG-010)
- [x] 补充单元测试与回归验证 (PRD-P2P-MIG-010)

## 依赖
- `doc/p2p/archive/distributed-consensus-membership-audit-revocation.prd.md`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/tests.rs`

## 状态
- 当前阶段：MR4 完成（吊销来源鉴权与审计落盘已落地）
- 后续进展：P3.17 已完成吊销授权治理与跨节点对账
- 最近更新：2026-02-10
