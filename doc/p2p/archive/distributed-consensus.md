> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式 Head 共识层（设计文档）

## 目标
- 在现有分布式存储/计算架构上补充 **Head 提交共识层**，避免单节点直接写入 world head。
- 让 `WorldHeadAnnounce` 在达到法定票数（quorum）前保持 `Pending`，只有 `Committed` 才写入 DHT。
- 保持与现有 `WorldBlock -> WorldHead` 数据链路兼容，不改变 CAS/分片/回放验证主流程。

## 范围

### In Scope（本次实现）
- 定义面向 Head 提交的 quorum 共识数据结构与状态机。
- 支持 proposer 自动投票、验证者投票、冲突提案拒绝、陈旧提案拒绝。
- 在共识提交后再执行 `put_world_head`，形成“先共识、后发布”的门控。
- 提供可复用的 runtime API 与单元测试。

### Out of Scope（本次不做）
- 拜占庭容错（BFT）与经济激励（staking/slashing）。
- 链上智能合约执行与代币记账。
- 动态验证者治理（加入/退出/轮换）与跨分片全局共识。
- 持久化 vote log（当前为进程内内存状态）。

## 接口 / 数据

### 核心类型
- `ConsensusConfig`
  - `validators: Vec<String>`：验证者集合。
  - `quorum_threshold: usize`：法定票数，`0` 表示默认多数派（`n/2+1`）。
- `ConsensusStatus`
  - `Pending | Committed | Rejected`。
- `ConsensusVote`
  - `validator_id/approve/reason/voted_at_ms`。
- `HeadConsensusRecord`
  - 记录某个 `(world_id,height)` 的提案、票据与状态。
- `ConsensusDecision`
  - 对外返回当前状态与票数统计。

### 共识引擎
- `QuorumConsensus::new(config)`
  - 规范化验证者列表；校验阈值合法且满足安全条件（`> half`）。
- `propose_head(head, proposer_id, proposed_at_ms)`
  - 校验 proposer 身份；拒绝陈旧高度；写入提案并记 proposer 赞成票。
- `vote_head(world_id, height, block_hash, validator_id, approve, voted_at_ms, reason)`
  - 校验验证者身份与 block_hash；处理重复票/冲突票；推进状态机。

### DHT 发布门控
- `propose_world_head_with_quorum(dht, consensus, head, proposer_id, proposed_at_ms)`
- `vote_world_head_with_quorum(dht, consensus, world_id, height, block_hash, validator_id, approve, voted_at_ms, reason)`
- 行为约束：仅当 `ConsensusStatus::Committed` 时调用 `dht.put_world_head(...)`。

### 状态机规则（摘要）
- `approvals >= quorum_threshold` => `Committed`。
- `rejections > validators_len - quorum_threshold` => `Rejected`。
- 同一 `(world_id,height)` 出现不同 `block_hash` => 冲突提案拒绝。
- 已有更高或同高已提交 head 时，拒绝陈旧提案。

## 里程碑
- **C1**：定义共识类型与阈值校验。
- **C2**：实现提案/投票/终态判定与冲突保护。
- **C3**：接入 DHT 发布门控、runtime 导出与单元测试。

## 风险
- 当前 vote/record 为内存态，进程重启后不会自动恢复。
- 非拜占庭场景设计，无法抵御恶意多数或签名伪造。
- 当前与租约/排序器只做“head 提交”层面的配合，后续仍需完善治理与成员管理。
