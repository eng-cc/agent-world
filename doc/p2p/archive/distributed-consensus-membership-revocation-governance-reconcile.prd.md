> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销授权治理与跨节点对账

## 1. Executive Summary
- Problem Statement: 在吊销同步链路中增加“授权治理”约束，避免仅“可信来源”但无治理授权的 requester 触发吊销。
- Proposed Solution: 提供跨节点 revoked key 集对账机制，识别并收敛节点间吊销状态漂移。
- Success Criteria:
  - SC-1: 保持与既有吊销广播/签名能力兼容，支持渐进启用策略。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销授权治理与跨节点对账 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 扩展 `MembershipRevocationSyncPolicy`：
  - AC-2: 新增 `authorized_requesters`
  - AC-3: `sync_key_revocations_with_policy(...)` 增加授权校验：
  - AC-4: requester 需满足 trusted 与 authorized 策略
  - AC-5: 新增对账 topic 与消息：
  - AC-6: `aw.<world_id>.membership.reconcile`
- Non-Goals:
  - 吊销授权与 lease holder/BFT 提案的强绑定。
  - 吊销对账结果的外部告警系统联动。
  - 对账结果写入独立分布式账本或数据库。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-governance-reconcile.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-governance-reconcile.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 吊销授权策略
- `MembershipRevocationSyncPolicy.authorized_requesters`
  - 为空：不启用显式授权名单
  - 非空：requester 必须命中名单

### 对账消息
- `MembershipRevocationCheckpointAnnounce`
  - `world_id`
  - `node_id`
  - `announced_at_ms`
  - `revoked_key_ids`
  - `revoked_set_hash`

### 对账策略与报告
- `MembershipRevocationReconcilePolicy`
  - `trusted_nodes`
  - `auto_revoke_missing_keys`
- `MembershipRevocationReconcileReport`
  - `drained`
  - `in_sync`
  - `diverged`
  - `merged`
  - `rejected`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计/项目文档完成。
  - **MR2**：授权治理策略实现与单测。
  - **MR3**：跨节点对账消息、策略、报告实现与单测。
  - **MR4**：回归验证与总文档/日志更新。
- Technical Risks:
  - 授权名单配置不一致会造成节点吊销处理分歧。
  - 自动收敛策略若配置不当，可能将异常远端状态扩散到本地。
  - 对账基于 gossip 广播存在时延窗口，需要配合周期策略持续收敛。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-032-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-032-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
