> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录快照密钥轮换与审计（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] 输出设计文档（`doc/p2p/archive/distributed-consensus-membership-rotation-audit.prd.md`） (PRD-P2P-MIG-034)
- [x] 增加 `signature_key_id` 并保持序列化兼容 (PRD-P2P-MIG-034)
- [x] 实现 `MembershipDirectorySignerKeyring` 多密钥签名/验签 (PRD-P2P-MIG-034)
- [x] 实现 key_id 轮换发布入口（active key） (PRD-P2P-MIG-034)
- [x] 扩展恢复策略（`require_signature_key_id` / `accepted_signature_key_ids`） (PRD-P2P-MIG-034)
- [x] 实现恢复审计报告结构与接口 (PRD-P2P-MIG-034)
- [x] 补充单元测试并执行分布式回归 (PRD-P2P-MIG-034)

## 依赖
- `doc/p2p/archive/distributed-consensus-membership-auth.prd.md`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`

## 状态
- 当前阶段：MR4 完成（密钥轮换与恢复审计已落地）
- 后续进展：P3.15 已完成审计持久化与密钥吊销传播
- 最近更新：2026-02-10
