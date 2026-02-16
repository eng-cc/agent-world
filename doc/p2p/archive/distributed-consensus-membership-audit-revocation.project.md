> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录审计持久化与吊销传播（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/p2p/distributed-consensus-membership-audit-revocation.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现 `MembershipAuditStore` 与内存实现
- [x] 实现 restore 审计持久化入口
- [x] 实现密钥吊销广播与订阅同步
- [x] 扩展 keyring 吊销状态与验签拦截
- [x] 扩展恢复策略吊销名单并补充单元测试
- [x] 执行格式化、单测与分布式回归

## 依赖
- `doc/p2p/distributed-consensus-membership-rotation-audit.md`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/tests.rs`

## 状态
- 当前阶段：MR4 完成（审计持久化与吊销传播已落地）
- 后续进展：P3.16 已完成吊销来源鉴权与审计落盘归档
- 最近更新：2026-02-10
