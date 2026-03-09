# Agent World Runtime：以太坊风格 PoS Head 共识

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 在现有 `QuorumConsensus` 基础上新增一套“类以太坊 PoS”的 Head 共识引擎。
- Proposed Solution: 将“按票数计数”升级为“按 stake 加权”的超级多数判定（默认 2/3）。
- Success Criteria:
  - SC-1: 引入 slot/epoch 与 proposer 选择约束，形成更接近 PoS 的提案与投票语义。
  - SC-2: 提供最小 slashing 规则（double vote / surround vote）用于防止明显违规投票。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：以太坊风格 PoS Head 共识 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增 `PosConsensus` 引擎（独立于 `QuorumConsensus`）。
  - AC-2: stake 加权投票判定：
  - AC-3: `approved_stake >= required_stake` => `Committed`
  - AC-4: `total_stake - rejected_stake < required_stake` => `Rejected`
  - AC-5: slot proposer 选择（按 stake 加权的确定性选择）。
  - AC-6: 最小 slashing 检查：
- Non-Goals:
  - 完整以太坊信标链流程（fork choice、execution payload、同步委员会、finalized checkpoint 全量语义）。
  - 真实 BLS 签名聚合与链上 slashing 惩罚执行。
  - 经济学参数（质押生命周期、退出队列、罚没资金流转）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distributed/distributed-pos-consensus.prd.md`
  - `doc/p2p/distributed/distributed-pos-consensus.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 核心类型
- `PosValidator { validator_id, stake }`
- `PosConsensusConfig`
  - `validators`
  - `supermajority_numerator`（默认 2）
  - `supermajority_denominator`（默认 3）
  - `epoch_length_slots`（默认 32）
- `PosConsensusStatus`
  - `Pending | Committed | Rejected`
- `PosAttestation`
  - `validator_id`
  - `approve`
  - `source_epoch`
  - `target_epoch`
  - `voted_at_ms`
  - `reason`
- `PosHeadRecord`
  - 记录 head、slot/epoch、attestation 集合与 stake 统计
- `PosConsensusDecision`
  - 对外返回状态与 stake 统计（approved/rejected/required）

### 主流程
- `propose_head(head, proposer_id, slot, proposed_at_ms)`
  - 校验 proposer 是有效 validator。
  - 校验 proposer 等于该 slot 的期望 proposer。
  - 记录提案并自动写入 proposer 的 approve attestation。
- `attest_head(...)`
  - 校验 validator 与提案匹配。
  - 执行 slashing 检查（double/surround）。
  - 写入 attestation 并重算 stake 判定状态。

### DHT 门控
- `propose_world_head_with_pos(dht, ...)`
- `attest_world_head_with_pos(dht, ...)`
- 仅 `Committed` 时执行 `dht.put_world_head(...)`。

## 5. Risks & Roadmap
- Phased Rollout:
  - POS-1：设计文档与项目管理文档落地。
  - POS-2：实现 `PosConsensus`、门控方法、快照持久化与单元测试。
  - POS-3：回归测试、文档状态收口与 devlog。
- Technical Risks:
  - 当前 proposer 选择为确定性加权方案，不包含链上随机性来源与抗操纵机制。
  - slashing 仅做“检测并拒绝”，不含经济惩罚与全网广播惩戒。
  - 与现有 node 主循环尚未接线，需要下一阶段接入执行路径。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-082-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-082-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
