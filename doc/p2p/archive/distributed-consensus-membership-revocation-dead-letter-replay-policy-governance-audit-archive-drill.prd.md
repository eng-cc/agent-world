> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计归档与恢复演练

## 1. Executive Summary
- Problem Statement: 为回退治理链路补齐治理级别审计归档，形成可追溯的治理历史。
- Proposed Solution: 提供恢复演练入口，验证重启后告警状态、治理状态与治理审计是否可恢复。
- Success Criteria:
  - SC-1: 保持现有审计/告警/治理入口兼容，新增“归档+演练”增强入口。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理审计归档与恢复演练 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增治理审计归档模型与归档存储抽象（内存/文件实现）。
  - AC-2: 新增治理联动执行增强入口：治理执行后自动归档治理审计记录。
  - AC-3: 新增恢复演练入口：读取 alert state、governance state 与治理审计历史并输出报告。
  - AC-4: 补充单元测试：归档落盘 round-trip、归档联动、恢复演练报告与参数校验。
- Non-Goals:
  - 外部审计平台（ELK/ClickHouse）适配。
  - 治理演练的自动定时触发与任务调度。
  - 跨节点统一治理审计聚合查询。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 治理审计归档记录（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord`
  - `world_id/node_id/audited_at_ms`
  - `governance_level`（`normal/stable/emergency`）
  - `rollback_streak`
  - `rolled_back`
  - `applied_policy`
  - `alert_emitted`

### 治理审计归档 store（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore`
  - `append(world_id, node_id, record)`
  - `list(world_id, node_id)`
- 实现：`InMemory...AuditStore`、`File...AuditStore`（JSONL）。

### 恢复演练报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport`
  - `world_id/node_id/drill_at_ms`
  - `alert_state`
  - `governance_state`
  - `recent_audits`
  - `has_emergency_history`

### 联动入口（拟）
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_governance_and_archive(...)`
- `run_revocation_dead_letter_replay_rollback_governance_recovery_drill(...)`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：治理审计归档模型与存储实现完成。
  - **MR3**：归档联动入口与恢复演练入口、测试完成。
  - **MR4**：总文档、项目状态、开发日志同步完成。
- Technical Risks:
  - 归档记录增长较快，后续需补充裁剪或分层归档策略。
  - 恢复演练报告字段过于简化时，可能不足以支撑复杂故障复盘。
  - 演练入口与真实恢复流程偏离时可能产生误判，需要保持语义对齐。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-020-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-020-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
