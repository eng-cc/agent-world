# Agent World Runtime：节点贡献积分多节点闭环测试

审计轮次: 3

## ROUND-002 主从口径
- 主入口文档：`doc/p2p/node/node-contribution-points.prd.md`。
- 本文档定位：仅记录多节点闭环测试的增量需求与验收口径；通用规格以主文档为准。

## 1. Executive Summary
- Problem Statement: 为“区块链 + P2P FS 场景下，提供算力和存储节点获得收益积分”补齐多节点闭环测试。
- Proposed Solution: 验证节点积分引擎在多节点输入下的关键经济语义：
- Success Criteria:
  - SC-1: 额外算力贡献获得收益；
  - SC-2: 存储贡献获得收益；
  - SC-3: 基础义务不足/低可靠性节点被惩罚；

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点贡献积分多节点闭环测试 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 基于 `NodePointsLedger` 构建多节点（>=3）两轮 epoch 的闭环测试。
  - AC-2: 覆盖 compute/storage/uptime/reliability/penalty 的组合场景。
  - AC-3: 验证结算输出：`distributed_points`、排序、惩罚效果、`cumulative_points`。
- Non-Goals:
  - 接入真实网络证明链路（PoSt/挑战响应）。
  - 与 viewer UI 的积分展示联动。
  - 代币化清算与兑换逻辑。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-contribution-points-multi-node-closure-test.prd.md`
  - `doc/p2p/node/node-contribution-points-multi-node-closure-test.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- 使用现有接口：
  - `NodePointsConfig`
  - `NodeContributionSample`
  - `NodePointsLedger::settle_epoch`
  - `EpochSettlementReport`
- 多节点样本组合：
  - 节点 A：高额外计算 + 中等存储 + 高在线；
  - 节点 B：中计算 + 高存储 + 高在线；
  - 节点 C：高计算但义务不足且惩罚高。

## 5. Risks & Roadmap
- Phased Rollout:
  - NCPM-1：补齐设计文档与项目管理文档。
  - NCPM-2：实现多节点闭环测试用例。
  - NCPM-3：执行 test_tier_required 回归并收口文档/devlog。
- Technical Risks:
  - 测试若依赖精确浮点值，未来调权重可能导致脆弱；应优先断言业务不变量（守恒、排名、惩罚生效、累计单调）。
  - 单一测试覆盖不足以替代真实网络闭环，后续仍需将贡献采样接入 runtime/node 实际数据面。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-089-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-089-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
