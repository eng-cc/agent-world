> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式 Head 共识快照持久化

## 1. Executive Summary
- Problem Statement: 为 `QuorumConsensus` 提供可落盘、可恢复的快照能力，解决进程重启后共识状态丢失的问题。
- Proposed Solution: 保证恢复后的提案/投票状态与阈值规则一致，避免把不安全记录重新加载进内存。
- Success Criteria:
  - SC-1: 在不引入完整链上治理的前提下，为后续成员治理与租约联动提供持久化基础。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式 Head 共识快照持久化 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 定义共识快照文件结构（版本号、验证者集合、阈值、记录列表）。
  - AC-2: 提供 `QuorumConsensus` 的快照保存与加载接口。
  - AC-3: 加载时校验快照合法性（版本、验证者身份、投票字段一致性），并重算记录终态。
  - AC-4: 提供单元测试覆盖快照 round-trip 和异常快照拒绝路径。
- Non-Goals:
  - 分布式复制存储与多副本容灾。
  - 动态验证者治理流程（提案/投票变更成员集合）。
  - 与 `LeaseManager` 的自动联动切换策略。
  - BFT/经济激励机制。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-persistence.prd.md`
  - `doc/p2p/archive/distributed-consensus-persistence.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 快照文件（JSON）
- 字段：
  - `version`: 快照版本（当前 `1`）
  - `validators`: 验证者集合
  - `quorum_threshold`: 法定票数
  - `records`: `HeadConsensusRecord` 列表
- 文件建议名：`consensus_snapshot.json`

### API
- `QuorumConsensus::save_snapshot_to_path(path)`
  - 将当前内存共识状态原子写入 JSON。
- `QuorumConsensus::load_snapshot_from_path(path)`
  - 从快照重建 `QuorumConsensus`。
- `QuorumConsensus::export_records()` / `import_records(...)`
  - 提供内存记录导入导出能力，便于后续接入外部 store。

### 校验规则（加载阶段）
- 快照版本必须匹配。
- `validators` 与 `quorum_threshold` 必须能通过 `QuorumConsensus::new` 安全校验。
- `record.proposer_id` 和每个 vote 的 `validator_id` 必须在验证者集合内。
- vote map key 必须与 vote 内部 `validator_id` 一致。
- 记录状态以票数重算（`Committed/Rejected/Pending`），不盲信快照中的原始状态字段。

## 5. Risks & Roadmap
- Phased Rollout:
  - **CP1**：定义快照文件结构与版本。
  - **CP2**：实现保存/加载 API 与校验逻辑。
  - **CP3**：补齐单元测试并通过回归。
- Technical Risks:
  - JSON 快照为单文件，极端大规模记录可能带来加载时延。
  - 当前仅本地持久化，不含跨节点防篡改机制。
  - 若验证者集合变化频繁，旧快照兼容策略需在后续成员治理中扩展。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-036-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-036-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
