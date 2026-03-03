> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位自动推进与空拉取幂等（P3.38）

## 1. Executive Summary
- Problem Statement: 在 P3.37 增量拉取基础上，补齐“查询 + 水位推进”一体化能力，降低消费端手工计算下一水位的重复逻辑。
- Proposed Solution: 明确空拉取场景的幂等语义：无新事件时保持水位不回退，便于轮询任务安全续拉。
- Success Criteria:
  - SC-1: 保持与现有 outcome 过滤和 `max_records` 限流策略兼容。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位自动推进与空拉取幂等（P3.38） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **包含**：
  - AC-2: 新增接口：
  - AC-3: `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_with_next_watermark(...)`
  - AC-4: 接口返回 `(events, next_since_event_at_ms)`，其中 `next_since_event_at_ms` 自动由本轮返回结果计算。
  - AC-5: 明确并实现水位单调性约束（`next_since_event_at_ms >= since_event_at_ms`）。
  - AC-6: 新增单元测试覆盖分批拉取与空拉取场景。
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`
  - `outcomes`
  - `max_records`
  - `event_bus`
- 输出：
  - `Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>`
  - `i64`（下一次增量拉取水位）
- 行为约束：
  - 仍复用既有增量逻辑：严格 `event_at_ms > since_event_at_ms`
  - 水位推进以本轮结果最后一条事件时间为准；若本轮无结果则保持原水位

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档与项目管理文档。
  - M2：实现增量拉取 + 下一水位联动接口。
  - M3：补充分批拉取与空拉取幂等测试。
  - M4：执行验证命令并同步总项目文档与 devlog。
- Technical Risks:
  - **同毫秒事件边界风险**：当前仅用时间水位，仍不完全覆盖同毫秒多事件稳定续拉；后续可引入复合游标。
  - **消费端误用风险**：若消费端不使用返回水位而自行计算，可能出现重复拉取或漏拉；需要在接口文档中明确推荐用法。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-028-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-028-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
