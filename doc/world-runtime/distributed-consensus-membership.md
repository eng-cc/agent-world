# Agent World Runtime：分布式 Head 共识成员治理与租约联动（设计文档）

## 目标
- 为 `QuorumConsensus` 增加可控的验证者成员治理能力（新增/移除/替换成员集合）。
- 建立与 `LeaseManager` 的联动约束：成员变更需由有效租约持有者发起。
- 保持 Head 共识安全性：成员变更后阈值仍满足 `> half`，并避免对进行中的提案造成不一致。

## 范围

### In Scope（本次实现）
- 定义成员变更请求与结果结构。
- 支持三类成员变更：`add_validator`、`remove_validator`、`replace_validators`。
- 新增租约授权入口：仅允许“当前有效租约 holder”执行成员变更。
- 提供“租约 holder 自动补齐为验证者”的辅助函数。
- 增加单元测试覆盖授权校验、阈值重算、进行中提案保护。

### Out of Scope（本次不做）
- 多轮链上治理投票流程（proposal/approval queue）。
- 跨节点一致成员目录同步协议。
- slashing、信誉权重、经济激励。

## 接口 / 数据

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

## 里程碑
- **CM1**：定义成员变更请求/结果数据结构。
- **CM2**：实现成员变更状态机与阈值重算。
- **CM3**：实现租约授权入口与自动补齐 helper。
- **CM4**：测试与文档更新。

## 风险
- 阻断“存在 Pending 提案时的成员变更”会降低变更灵活性，但可显著降低一致性风险。
- 当前租约授权只校验本地租约对象，跨节点租约真值仍依赖上层协调。
- 保守阈值策略可能让小规模集群在缩容后阈值偏高，需要后续治理流程做细化调参。
