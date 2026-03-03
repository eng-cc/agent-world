> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式 Head 共识层

## 1. Executive Summary
- Problem Statement: 在现有分布式存储/计算架构上补充 **Head 提交共识层**，避免单节点直接写入 world head。
- Proposed Solution: 让 `WorldHeadAnnounce` 在达到法定票数（quorum）前保持 `Pending`，只有 `Committed` 才写入 DHT。
- Success Criteria:
  - SC-1: 保持与现有 `WorldBlock -> WorldHead` 数据链路兼容，不改变 CAS/分片/回放验证主流程。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式 Head 共识层 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 定义面向 Head 提交的 quorum 共识数据结构与状态机。
  - AC-2: 支持 proposer 自动投票、验证者投票、冲突提案拒绝、陈旧提案拒绝。
  - AC-3: 在共识提交后再执行 `put_world_head`，形成“先共识、后发布”的门控。
  - AC-4: 提供可复用的 runtime API 与单元测试。
- Non-Goals:
  - 拜占庭容错（BFT）与经济激励（staking/slashing）。
  - 链上智能合约执行与代币记账。
  - 动态验证者治理（加入/退出/轮换）与跨分片全局共识。
  - 持久化 vote log（当前为进程内内存状态）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus.prd.md`
  - `doc/p2p/archive/distributed-consensus.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - **C1**：定义共识类型与阈值校验。
  - **C2**：实现提案/投票/终态判定与冲突保护。
  - **C3**：接入 DHT 发布门控、runtime 导出与单元测试。
- Technical Risks:
  - 当前 vote/record 为内存态，进程重启后不会自动恢复。
  - 非拜占庭场景设计，无法抵御恶意多数或签名伪造。
  - 当前与租约/排序器只做“head 提交”层面的配合，后续仍需完善治理与成员管理。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-038-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-038-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
