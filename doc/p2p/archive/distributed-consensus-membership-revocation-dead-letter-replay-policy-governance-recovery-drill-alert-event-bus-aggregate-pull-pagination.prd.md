> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线聚合拉取与分页增量查询（P3.36）

## 1. Executive Summary
- Problem Statement: 在 P3.35 事件总线写入能力基础上，补齐读侧能力，支持跨节点聚合拉取恢复演练告警事件。
- Proposed Solution: 提供统一的时间窗过滤、outcome 过滤与分页增量查询能力，降低上层治理面板/巡检任务的重复聚合成本。
- Success Criteria:
  - SC-1: 保持现有事件总线实现（内存/文件）兼容，不引入破坏性接口变更。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线聚合拉取与分页增量查询（P3.36） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **包含**：
  - AC-2: 新增 `MembershipSyncClient` 事件总线聚合查询入口：
  - AC-3: `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated(...)`
  - AC-4: 支持过滤维度：
  - AC-5: `min_event_at_ms`（时间下界）
  - AC-6: `outcomes`（按 `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome` 过滤）
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 查询输入：
  - `world_id`
  - `node_ids`
  - `min_event_at_ms`
  - `outcomes`
  - `offset`
  - `max_records`
  - `event_bus`
- 查询输出：
  - `Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>`
- 行为约束：
  - `node_ids` 不能为空；
  - `max_records` 必须大于 0；
  - 聚合结果按 `event_at_ms desc` + `node_id asc` 稳定排序；
  - 聚合时仅保留与查询维度一致的 `world_id/node_id` 事件记录。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档与项目管理文档。
  - M2：完成 `replay_archive_federated.rs` 聚合拉取实现与参数校验。
  - M3：完成 `recovery_replay_federated_tests.rs` 单元测试覆盖（过滤、分页、异常参数）。
  - M4：运行格式化、定向测试与模块回归，更新总项目管理文档与 devlog。
- Technical Risks:
  - **事件量增长风险**：当前文件事件总线 `list` 读全量后过滤，超大规模下会有内存与延迟压力；后续可补按范围读取接口。
  - **过滤歧义风险**：若写入端出现错误 `world_id/node_id`，读侧需显式过滤并保守忽略异常事件，避免跨节点污染。
  - **分页一致性风险**：基于偏移分页在并发写入场景可能出现“翻页漂移”；后续可引入游标（事件时间+序号）改善一致性。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-023-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-023-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
