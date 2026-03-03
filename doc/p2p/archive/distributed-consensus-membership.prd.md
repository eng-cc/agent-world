> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式 Head 共识成员治理与租约联动

## 1. Executive Summary
- Problem Statement: 为 `QuorumConsensus` 增加可控的验证者成员治理能力（新增/移除/替换成员集合）。
- Proposed Solution: 建立与 `LeaseManager` 的联动约束：成员变更需由有效租约持有者发起。
- Success Criteria:
  - SC-1: 保持 Head 共识安全性：成员变更后阈值仍满足 `> half`，并避免对进行中的提案造成不一致。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式 Head 共识成员治理与租约联动 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 定义成员变更请求与结果结构。
  - AC-2: 支持三类成员变更：`add_validator`、`remove_validator`、`replace_validators`。
  - AC-3: 新增租约授权入口：仅允许“当前有效租约 holder”执行成员变更。
  - AC-4: 提供“租约 holder 自动补齐为验证者”的辅助函数。
  - AC-5: 增加单元测试覆盖授权校验、阈值重算、进行中提案保护。
- Non-Goals:
  - 多轮链上治理投票流程（proposal/approval queue）。
  - 跨节点一致成员目录同步协议。
  - slashing、信誉权重、经济激励。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 成员变更数据结构
- `ConsensusMembershipChange`
  - `AddValidator { validator_id }`
  - `RemoveValidator { validator_id }`
  - `ReplaceValidators { validators, quorum_threshold }`
- `ConsensusMembershipChangeRequest`
  - `requester_id`、`requested_at_ms`、`reason`、`change`
- `ConsensusMembershipChangeResult`
  - `applied`、`validators`、`quorum_threshold`

### 核心 API
- `QuorumConsensus::apply_membership_change(request)`
  - 处理成员变更并返回结果。
- `QuorumConsensus::apply_membership_change_with_lease(request, lease)`
  - 在成员变更前校验租约 holder 身份与时效。
- `ensure_lease_holder_validator(consensus, lease, requested_at_ms)`
  - 当租约持有者尚未在 validator 集合中时，自动补齐。

### 约束规则
- 若存在 `Pending` 提案，拒绝成员变更（避免进行中提案在变更期间语义漂移）。
- 租约联动入口要求：
  - `lease` 必须存在。
  - `lease.holder_id == requester_id`。
  - `lease.expires_at_ms > requested_at_ms`。
- 阈值策略：
  - 成员增减时按“保守阈值”重算：维持不低于当前阈值且满足多数安全约束。
  - `replace_validators` 时按请求阈值（`0` 表示多数默认）重新校验。

## 5. Risks & Roadmap
- Phased Rollout:
  - **CM1**：定义成员变更请求/结果数据结构。
  - **CM2**：实现成员变更状态机与阈值重算。
  - **CM3**：实现租约授权入口与自动补齐 helper。
  - **CM4**：测试与文档更新。
- Technical Risks:
  - 阻断“存在 Pending 提案时的成员变更”会降低变更灵活性，但可显著降低一致性风险。
  - 当前租约授权只校验本地租约对象，跨节点租约真值仍依赖上层协调。
  - 保守阈值策略可能让小规模集群在缩容后阈值偏高，需要后续治理流程做细化调参。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-035-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-035-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
