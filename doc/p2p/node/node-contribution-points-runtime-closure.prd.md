# Agent World Runtime：节点贡献积分运行时闭环

## 1. Executive Summary
- Problem Statement: 在运行时侧形成 Node Contribution Points 的闭环：采样、结算、审计、回放一致。
- Proposed Solution: 为后续节点收益、治理参数调优、发布门禁提供统一口径。
- Success Criteria:
  - SC-1: 完成 strict 6 章重写并保持语义保真。
  - SC-2: 任务/依赖/状态与 PRD-ID 可追溯。
  - SC-3: 治理检查通过且引用可达。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点贡献积分运行时闭环 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 运行时采样器（snapshot/storage -> epoch settlement）。
  - AC-2: 多节点闭环验证与 `test_tier_required` 回归口径。
  - AC-3: 与既有 `node-contribution-points` 设计的运行时接线收口。
- Non-Goals:
  - 新经济模型设计。
  - 跨链或外部记账系统接入。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-contribution-points-runtime-closure.prd.md`
  - `doc/p2p/node/node-contribution-points-runtime-closure.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 运行时模块：`crates/agent_world/src/runtime/node_points.rs`
- 运行时接线：`crates/agent_world/src/runtime/mod.rs`
- 节点快照输入：`crates/agent_world_node/src/lib.rs`（`NodeSnapshot`）
- 关联设计：`doc/p2p/node/node-contribution-points.prd.md`

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成运行时采样器接线。
  - M2：完成多节点闭环测试。
  - M3：完成 required 回归与文档收口。
- Technical Risks:
  - 采样窗口与结算窗口错配可能导致奖励偏差。
  - 快照不一致会影响回放可重复性与审计可解释性。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-090-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-090-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
