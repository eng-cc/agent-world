# Agent World Runtime：分布式 Head 共识层（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus.md`）
- [x] 新增 quorum 共识模块（`crates/agent_world/src/runtime/distributed_consensus.rs`）
- [x] 实现提案/投票状态机（Pending/Committed/Rejected）
- [x] 实现冲突提案与陈旧提案保护
- [x] 接入 DHT 发布门控（提交后才更新 world head）
- [x] runtime/lib 对外导出共识 API
- [x] 运行共识与分布式回归测试并记录结果

## 依赖
- `doc/world-runtime/distributed-runtime.md`
- `crates/agent_world/src/runtime/distributed.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`
- `crates/agent_world/src/runtime/distributed_index.rs`

## 状态
- 当前阶段：C3 完成（已通过共识与分布式回归测试）
- 下一步：C4（共识持久化与成员治理）
- 最近更新：2026-02-10
