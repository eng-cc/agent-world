# Agent World Runtime：分布式成员目录同步与变更广播（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-sync.md`）
- [x] 新增成员目录 topic 与 helper
- [x] 实现成员目录广播消息结构
- [x] 实现发布/订阅/同步客户端（`MembershipSyncClient`）
- [x] 实现目录同步幂等处理（applied/ignored）
- [x] 运行单元测试与分布式回归验证并记录结果

## 依赖
- `doc/world-runtime/distributed-consensus-membership.md`
- `crates/agent_world/src/runtime/distributed.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_consensus.rs`

## 状态
- 当前阶段：CS4 完成（成员目录同步与广播已落地）
- 后续关联：P3.12 已补齐成员目录 DHT 快照与恢复（见 `distributed-consensus-membership-dht.md`）
- 最近更新：2026-02-10
