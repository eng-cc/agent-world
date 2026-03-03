> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合序号游标状态持久化（P3.41）

## 1. Executive Summary
- Problem Statement: 在 P3.40 复合序号游标查询能力基础上，补齐消费端游标状态持久化能力，支持进程重启后续拉。
- Proposed Solution: 提供“读取状态 -> 拉取增量 -> 更新状态”一体化查询入口，降低业务侧重复样板代码。
- Success Criteria:
  - SC-1: 保持与既有 outcome 过滤、`max_records` 限流和空拉取幂等行为兼容。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合序号游标状态持久化（P3.41） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **包含**：
  - AC-2: 新增复合序号游标状态模型与 store 抽象。
  - AC-3: 新增内存/文件两种状态存储实现。
  - AC-4: 新增带状态存储的一体化增量续拉接口。
  - AC-5: 新增单元测试覆盖：首次拉取、续拉、空拉取幂等、文件存储回读。
  - AC-6: **不包含**：
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 状态模型：
  - `world_id`
  - `consumer_id`
  - `since_event_at_ms`
  - `since_node_id`
  - `since_node_event_offset`
- 新增接口（示意）：
  - `query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_with_composite_sequence_cursor_state(...)`
- 行为语义：
  - 若 store 无状态，使用调用方提供的初始 cursor。
  - 查询后将返回 cursor 持久化到 store。
  - 空拉取时 cursor 保持不变并持久化（幂等）。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档与项目管理文档。
  - M2：实现游标状态模型与内存/文件 store。
  - M3：实现带状态存储的一体化续拉接口。
  - M4：补充测试、完成验证与文档/devlog 同步。
- Technical Risks:
  - **并发覆盖风险**：多消费者共用同一 `consumer_id` 会互相覆盖 cursor；需业务侧明确 consumer 维度隔离。
  - **文件写放大风险**：高频轮询会触发频繁落盘；后续可评估批量 flush 或节流策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-026-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-026-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
