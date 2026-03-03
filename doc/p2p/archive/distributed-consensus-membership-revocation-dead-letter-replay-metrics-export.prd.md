> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放调度与指标导出

## 1. Executive Summary
- Problem Statement: 为成员目录吊销告警死信队列提供可调度回放能力，将可恢复死信重新注入恢复队列。
- Proposed Solution: 为告警投递指标提供统一导出能力（内存/文件），便于后续对接监控与离线分析。
- Success Criteria:
  - SC-1: 保持现有 ACK 重试与死信归档链路兼容，支持增量启用。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放调度与指标导出 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 扩展 `MembershipRevocationAlertDeadLetterStore`：支持 list/replace 与 delivery metrics append/list。
  - AC-2: 补齐内存/文件 dead-letter store 对应实现。
  - AC-3: 新增 `replay_revocation_dead_letters(...)`：按上限批量回放 dead-letter 到 recovery pending 队列。
  - AC-4: 新增 `run_revocation_dead_letter_replay_schedule(...)`：按间隔调度 dead-letter 回放。
  - AC-5: 新增 `export_revocation_alert_delivery_metrics(...)` 与协同调度联动导出入口。
  - AC-6: 补充单元测试覆盖回放顺序、调度触发与指标导出。
- Non-Goals:
  - 基于告警等级/错误码的差异化回放策略。
  - 死信回放去重与跨节点回放协调锁。
  - 对接外部监控协议（Prometheus/OpenTelemetry）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### Dead-letter store 扩展
- `list(world_id, node_id)`
- `replace(world_id, node_id, records)`
- `append_delivery_metrics(world_id, node_id, exported_at_ms, metrics)`
- `list_delivery_metrics(world_id, node_id)`

### 回放入口
- `replay_revocation_dead_letters(...)`
  - 从 dead-letter store 拉取记录
  - 取前 N 条回放到 pending 队列
  - 余量回写 dead-letter store

### 调度入口
- `run_revocation_dead_letter_replay_schedule(...)`
  - 基于 `replay_interval_ms` 判定是否到期
  - 到期时执行 replay 并更新 `last_replay_at_ms`

### 指标导出
- `export_revocation_alert_delivery_metrics(...)`
  - 将 `MembershipRevocationAlertDeliveryMetrics` 导出到 store
- `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter_and_metrics_export(...)`
  - 协同执行后自动导出本次 delivery metrics

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目文档完成。
  - **MR2**：dead-letter store 扩展与实现完成。
  - **MR3**：回放调度与指标导出入口完成。
  - **MR4**：测试、总文档、项目状态同步完成。
- Technical Risks:
  - 回放策略当前按 FIFO + fixed limit，缺少优先级控制。
  - 指标导出与主流程共享存储时，写入失败可能影响主流程可观测性。
  - 大规模回放可能与在线新告警竞争 recovery 队列容量。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-014-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-014-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
