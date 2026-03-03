> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计归档保留策略与演练调度

## 1. Executive Summary
- Problem Statement: 为治理审计归档增加可配置保留策略，控制历史记录增长并保留关键窗口数据。
- Proposed Solution: 提供可重复执行的归档裁剪入口，支持按条数与时间窗口双维治理。
- Success Criteria:
  - SC-1: 为恢复演练新增调度编排能力，支持周期化演练与状态持久化。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理审计归档保留策略与演练调度 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增治理审计归档保留策略模型（按最大记录数、最大保留时长）。
  - AC-2: 新增治理审计归档裁剪入口与执行报告。
  - AC-3: 新增恢复演练调度策略、调度状态存储（内存/文件）与调度编排入口。
  - AC-4: 补充单元测试：裁剪策略、文件裁剪、调度间隔门控、参数校验。
- Non-Goals:
  - 多 world 的统一归档策略中心化管理。
  - 审计记录冷热分层、对象存储下沉。
  - 外部任务系统（cron/k8s job）联动。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 归档保留策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy`
  - `max_records: usize`（最多保留记录数）
  - `max_age_ms: i64`（最大保留时长，基于 `audited_at_ms`）

### 归档裁剪报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditPruneReport`
  - `world_id/node_id/pruned_at_ms`
  - `before/after`
  - `pruned_by_age/pruned_by_capacity`

### 演练调度策略与状态（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy`
  - `drill_interval_ms`
  - `recent_audit_limit`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleState`
  - `last_drill_at_ms`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore`
  - `load/save`（内存/文件实现）

### 调度运行报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduledRunReport`
  - `drill_due`
  - `drill_executed`
  - `drill_report`（可选，复用 drill 报告）

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：归档保留策略与裁剪入口实现完成。
  - **MR3**：恢复演练调度与状态存储实现完成。
  - **MR4**：测试、总文档、项目状态、devlog 同步完成。
- Technical Risks:
  - 裁剪策略过于激进时，可能影响事后审计深度。
  - 时间窗口依赖 `now_ms`，时钟漂移可能影响裁剪与调度边界。
  - 调度状态与业务状态更新时序不一致时，可能出现重复演练或漏演练。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-021-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-021-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
