> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位拉取与 outcome 汇总（P3.37）

## 1. Executive Summary
- Problem Statement: 在 P3.36 分页聚合查询基础上，补齐“增量消费友好”的查询能力，支持按时间水位做严格增量拉取。
- Proposed Solution: 提供 outcome 维度的聚合汇总，便于巡检任务与监控面板快速判断告警结构变化。
- Success Criteria:
  - SC-1: 保持事件总线读写接口兼容，不引入存储格式变更。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位拉取与 outcome 汇总（P3.37） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **包含**：
  - AC-2: 新增增量拉取入口：
  - AC-3: `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since(...)`
  - AC-4: 新增 outcome 聚合入口：
  - AC-5: `summarize_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated_by_outcome(...)`
  - AC-6: 复用并抽取聚合查询公共读取逻辑，统一 `world/node` 过滤一致性。
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 增量拉取输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`（严格 `>`）
  - `outcomes`（可选过滤）
  - `max_records`
- 增量拉取输出：
  - `Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>`
  - 结果按 `event_at_ms asc + node_id asc` 排序，便于消费端顺序处理。
- outcome 汇总输出：
  - `BTreeMap<String, usize>`，key 使用稳定标签：
    - `emitted`
    - `suppressed_cooldown`
    - `suppressed_no_anomaly`
    - `skipped_no_drill`

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档与项目管理文档。
  - M2：完成 federated 查询实现（增量水位拉取 + outcome 汇总）。
  - M3：完成单元测试覆盖（严格水位、过滤、limit、汇总）。
  - M4：完成验证、项目总文档与 devlog 更新。
- Technical Risks:
  - **同毫秒并发事件风险**：纯时间水位无法完全区分同毫秒事件顺序；后续可引入复合游标（时间+节点+序号）。
  - **读放大风险**：当前读取路径仍基于事件总线 `list` 后过滤，极端高基数场景仍有扫描开销。
  - **汇总标签扩展风险**：若新增 outcome 枚举，需要同步更新标签映射与统计逻辑，避免遗漏分类。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-029-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-029-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
