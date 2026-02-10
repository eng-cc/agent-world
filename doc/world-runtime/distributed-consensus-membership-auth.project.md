# Agent World Runtime：成员目录快照签名与来源校验（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-auth.md`）
- [x] 扩展成员目录快照/广播结构支持可选签名字段
- [x] 实现 `MembershipDirectorySigner`（签名/验签）
- [x] 实现 `publish_membership_change_with_dht_signed` 发布链路
- [x] 实现 `restore_membership_from_dht_verified` 来源与签名校验
- [x] 补充单元测试并执行回归验证

## 依赖
- `doc/world-runtime/distributed-consensus-membership-dht.md`
- `crates/agent_world/src/runtime/distributed_dht.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`

## 状态
- 当前阶段：MA4 完成（签名与来源校验已落地）
- 下一步：评估非对称签名与密钥轮换策略
- 最近更新：2026-02-10
