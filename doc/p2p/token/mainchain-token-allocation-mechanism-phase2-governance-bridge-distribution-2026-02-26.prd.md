# Agent World 主链 Token 分配机制二期：地址绑定 + 治理绑定 + 分发闭环（2026-02-26）

- 对应设计文档: `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.design.md`
- 对应项目管理文档: `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将 NodePoints -> 主链 Token 桥接的收款方从 `account_id = node_id` 升级为真实地址绑定语义。
- Proposed Solution: 将 `Action::UpdateMainTokenPolicy` 与治理提案生命周期绑定，避免“脱离治理状态”的参数更新。
- Success Criteria:
  - SC-1: 为 `staking_reward_pool`、`ecosystem_pool`、`security_reserve` 提供可审计的分发闭环，避免只入 treasury 无后续主路径动作。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World 主链 Token 分配机制二期：地址绑定 + 治理绑定 + 分发闭环（2026-02-26） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: Runtime 状态新增：节点主链地址绑定、主链 treasury 分发审计记录。
  - AC-2: Runtime Action/DomainEvent 新增：主链 treasury 分发动作与事件。
  - AC-3: Runtime 规则增强：
  - AC-4: 主链策略更新必须绑定到已批准/已应用的治理提案；
  - AC-5: NodePoints 桥接出账改为使用“节点绑定主链地址”；
  - AC-6: treasury 分发动作支持治理绑定和幂等审计。
- Non-Goals:
  - 跨链地址体系、外部钱包协议（只做 runtime 内部地址字符串绑定）。
  - 治理模块（manifest governance / gameplay governance）结构重构。
  - 新增经济学参数（通胀公式和 split 比例不变）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`
  - `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) 状态新增
- `WorldState.node_main_token_account_bindings: BTreeMap<String, String>`
  - key: `node_id`
  - value: `main_token_account_id`
- `WorldState.main_token_treasury_distribution_records: BTreeMap<String, MainTokenTreasuryDistributionRecord>`
  - key: `distribution_id`

### 2) 主链地址绑定
- `World::bind_node_identity(node_id, public_key_hex)` 保持可用，同时补齐主链地址绑定默认派生。
- 新增查询/写接口：
  - `World::node_main_token_account(node_id)`
  - `World::bind_node_main_token_account(node_id, account_id)`
- 桥接分配时，优先使用 `node_main_token_account_bindings[node_id]`。

### 3) 治理绑定（主链策略更新）
- `Action::UpdateMainTokenPolicy { proposal_id, next }` 规则增强：
  - `proposal_id` 必须存在于 `World.proposals`；
  - 对应提案状态必须为 `ProposalStatus::Approved` 或 `ProposalStatus::Applied`。
- 原有参数边界校验和延迟生效规则保持不变。

### 4) treasury 分发闭环
- 新增动作：
```rust
Action::DistributeMainTokenTreasury {
  proposal_id: ProposalId,
  distribution_id: String,
  bucket_id: String,
  distributions: Vec<MainTokenTreasuryDistribution>,
}
```
- 新增事件：
```rust
DomainEvent::MainTokenTreasuryDistributed {
  proposal_id: ProposalId,
  distribution_id: String,
  bucket_id: String,
  total_amount: u64,
  distributions: Vec<MainTokenTreasuryDistribution>,
}
```
- 允许分发 bucket：
  - `staking_reward_pool`
  - `ecosystem_pool`
  - `security_reserve`
- 状态应用：
  - 扣减 treasury bucket；
  - 增加目标账户 `liquid_balance`；
  - 增加 `circulating_supply`（`total_supply` 不变）；
  - 写入 `main_token_treasury_distribution_records`，按 `distribution_id` 幂等防重。

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：设计/项目文档建档。
  - M1：节点主链地址绑定接线 + NodePoints 桥接改造。
  - M2：主链策略更新治理绑定落地。
  - M3：treasury 分发动作/事件/状态闭环落地。
  - M4：测试矩阵与文档回写收口。
- Technical Risks:
  - 兼容性风险：旧快照缺少新增状态字段，需要 `serde(default)` 保证兼容。
  - 治理流程风险：若环境中未走 `proposals` 流程，策略更新会被拒绝（需测试覆盖和文档提示）。
  - 经济风险：treasury 分发动作如果无治理绑定将存在滥发风险，本期通过 `proposal_id` 绑定降低风险。
  - 地址质量风险：当前地址格式为 runtime 内字符串，不等价链外钱包地址。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-111-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-111-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
