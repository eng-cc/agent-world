# Agent World Runtime：分布式 Head 共识成员治理与租约联动（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership.md`）
- [x] 定义成员变更请求/结果结构
- [x] 实现成员增减/替换接口
- [x] 实现进行中提案保护（Pending 阻断）
- [x] 实现租约授权入口（holder + 时效校验）
- [x] 实现租约 holder 自动补齐 helper
- [x] 运行单元测试与分布式回归验证并记录结果

## 依赖
- `doc/world-runtime/distributed-consensus.md`
- `doc/world-runtime/distributed-consensus-persistence.md`
- `crates/agent_world/src/runtime/distributed_consensus.rs`
- `crates/agent_world/src/runtime/distributed_lease.rs`

## 状态
- 当前阶段：CM4 完成（成员治理与租约联动已落地）
- 下一步：推进跨节点成员目录同步策略
- 最近更新：2026-02-10
