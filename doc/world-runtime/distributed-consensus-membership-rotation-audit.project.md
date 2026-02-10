# Agent World Runtime：成员目录快照密钥轮换与审计（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-rotation-audit.md`）
- [x] 增加 `signature_key_id` 并保持序列化兼容
- [x] 实现 `MembershipDirectorySignerKeyring` 多密钥签名/验签
- [x] 实现 key_id 轮换发布入口（active key）
- [x] 扩展恢复策略（`require_signature_key_id` / `accepted_signature_key_ids`）
- [x] 实现恢复审计报告结构与接口
- [x] 补充单元测试并执行分布式回归

## 依赖
- `doc/world-runtime/distributed-consensus-membership-auth.md`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`

## 状态
- 当前阶段：MR4 完成（密钥轮换与恢复审计已落地）
- 后续进展：P3.15 已完成审计持久化与密钥吊销传播
- 最近更新：2026-02-10
