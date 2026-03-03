> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计聚合查询与演练告警事件总线

## 1. Executive Summary
- Problem Statement: 补齐治理审计冷热归档的跨节点聚合查询能力，支持故障复盘时一次性查看多节点历史。
- Proposed Solution: 将恢复演练告警结果标准化为事件并写入事件总线，形成“判定→告警→事件追踪”闭环。
- Success Criteria:
  - SC-1: 在保持现有归档分层与演练告警流程兼容的前提下，新增可选事件总线联动入口。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理审计聚合查询与演练告警事件总线 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增治理审计归档聚合查询策略与查询报告（支持 hot/cold 分层、时间窗、级别过滤、条数限制）。
  - AC-2: 新增跨节点聚合查询入口，支持按 node 列表聚合并按时间排序返回。
  - AC-3: 新增恢复演练告警事件模型与事件总线抽象（内存/文件实现）。
  - AC-4: 新增“归档分层+演练告警+事件总线”联动入口，执行后同步落盘事件。
  - AC-5: 补充单元测试：聚合过滤/排序、事件总线落盘 round-trip、联动入口事件写入。
- Non-Goals:
  - 冷层对象存储（S3/OSS）适配与远端生命周期治理。
  - 跨进程消息队列（Kafka/NATS）投递与 ACK 重试语义。
  - 对外通知适配器（Webhook/IM）细分路由策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 聚合查询策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy`
  - `include_hot`
  - `include_cold`
  - `max_records`
  - `min_audited_at_ms`
  - `levels`

### 聚合查询结果（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord`
  - `world_id/node_id`
  - `tier`（`hot/cold`）
  - `audit`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport`
  - `world_id/queried_node_count`
  - `scanned_hot/scanned_cold`
  - `returned`
  - `records`

### 演练告警事件总线（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent`
  - `world_id/node_id/event_at_ms`
  - `outcome`（`emitted/suppressed_cooldown/suppressed_no_anomaly/skipped_no_drill`）
  - `reasons`
  - `severity`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus`
  - `publish(...)`
  - `list(world_id, node_id)`
- 实现：`InMemory...EventBus`、`File...EventBus`（JSONL）。

### 联动入口（拟）
- `run_revocation_dead_letter_replay_rollback_governance_archive_tiered_offload_with_drill_schedule_alert_and_event_bus(...)`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：聚合查询策略、报告与查询入口实现完成。
  - **MR3**：演练告警事件总线（内存/文件）与事件映射实现完成。
  - **MR4**：联动入口、测试、总文档与 devlog 同步完成。
- Technical Risks:
  - 聚合查询在节点数较多时会增加扫描成本，后续可能需要分页与索引。
  - hot/cold 同时查询时可能出现重复语义，需要明确 tier 与去重策略。
  - 事件总线先落本地文件，跨进程消费一致性需后续阶段补齐。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-019-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-019-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
