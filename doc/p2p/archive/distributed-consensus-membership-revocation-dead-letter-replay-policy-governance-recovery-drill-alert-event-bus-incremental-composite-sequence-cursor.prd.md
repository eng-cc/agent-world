> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标序号续拉（P3.40）

## 1. Executive Summary
- Problem Statement: 在 P3.39 “时间水位 + 节点游标”基础上，补齐同节点同毫秒多事件的稳定续拉能力。
- Proposed Solution: 增加序号级游标维度，避免 `max_records` 分页时出现“同节点同毫秒后续事件漏拉”。
- Success Criteria:
  - SC-1: 保持与既有 outcome 过滤、`max_records` 限流和空拉取幂等行为兼容。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标序号续拉（P3.40） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **包含**：
  - AC-2: 新增接口：
  - AC-3: `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor(...)`
  - AC-4: 支持输入 cursor：
  - AC-5: `since_event_at_ms`
  - AC-6: `since_node_id`
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 输入：
  - `world_id`、`node_ids`
  - `since_event_at_ms`
  - `since_node_id`
  - `since_node_event_offset`
  - `outcomes`
  - `max_records`
  - `event_bus`
- 排序与过滤语义（复合键）：
  - `(event_at_ms, node_id, node_event_offset)` 严格大于 cursor 才返回
  - 其中 `node_event_offset` 为“同一 node_id 内原始事件列表顺序索引”
- 输出：
  - `(Vec<Event>, i64, Option<String>, usize)`
  - 空结果时保持输入 cursor 不变（幂等续拉）

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档与项目管理文档。
  - M2：完成序号级复合游标查询实现。
  - M3：完成同节点同毫秒分页续拉测试。
  - M4：完成验证、总项目管理文档与 devlog 更新。
- Technical Risks:
  - **顺序语义风险**：`node_event_offset` 依赖事件总线 list 顺序稳定；若底层实现改为无序存储，会破坏 cursor 一致性。
  - **跨实现一致性风险**：内存与文件事件总线需保证 list 顺序语义一致（当前均为 append 顺序）。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-027-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-027-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
