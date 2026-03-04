> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式 Head 共识成员治理与租约联动（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] 输出设计文档（`doc/p2p/archive/distributed-consensus-membership.prd.md`） (PRD-P2P-MIG-035)
- [x] 定义成员变更请求/结果结构 (PRD-P2P-MIG-035)
- [x] 实现成员增减/替换接口 (PRD-P2P-MIG-035)
- [x] 实现进行中提案保护（Pending 阻断） (PRD-P2P-MIG-035)
- [x] 实现租约授权入口（holder + 时效校验） (PRD-P2P-MIG-035)
- [x] 实现租约 holder 自动补齐 helper (PRD-P2P-MIG-035)
- [x] 运行单元测试与分布式回归验证并记录结果 (PRD-P2P-MIG-035)

## 依赖
- `doc/p2p/archive/distributed-consensus.prd.md`
- `doc/p2p/archive/distributed-consensus-persistence.prd.md`
- `crates/agent_world/src/runtime/distributed_consensus.rs`
- `crates/agent_world/src/runtime/distributed_lease.rs`

## 状态
- 当前阶段：CM4 完成（成员治理与租约联动已落地）
- 下一步：推进跨节点成员目录同步策略
- 最近更新：2026-02-10
