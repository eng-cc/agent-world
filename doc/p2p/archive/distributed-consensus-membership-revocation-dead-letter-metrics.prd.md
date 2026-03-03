> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销告警恢复死信归档与投递指标

## 1. Executive Summary
- Problem Statement: 为成员目录吊销告警恢复链路补齐死信归档机制，避免重试耗尽/容量裁剪时告警静默丢失。
- Proposed Solution: 为告警投递过程提供结构化指标（attempt/success/failure/deferred/drop），提升恢复链路可观测性。
- Success Criteria:
  - SC-1: 与现有恢复队列、ACK 重试、协同调度接口保持兼容，支持增量接入。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销告警恢复死信归档与投递指标 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增死信归档模型与存储抽象：
  - AC-2: `MembershipRevocationAlertDeadLetterReason`
  - AC-3: `MembershipRevocationAlertDeadLetterRecord`
  - AC-4: `MembershipRevocationAlertDeadLetterStore`
  - AC-5: `InMemoryMembershipRevocationAlertDeadLetterStore`
  - AC-6: `FileMembershipRevocationAlertDeadLetterStore`
- Non-Goals:
  - 死信再投递调度器（自动 replay DLQ）。
  - 死信内容脱敏/加密策略与审计访问控制。
  - Prometheus/OpenTelemetry exporter 适配。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-metrics.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-metrics.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 死信记录
- `MembershipRevocationAlertDeadLetterRecord`
  - `world_id`
  - `node_id`
  - `dropped_at_ms`
  - `reason`
  - `pending_alert`

### 死信原因
- `MembershipRevocationAlertDeadLetterReason`
  - `retry_limit_exceeded`
  - `capacity_evicted`

### 投递指标
- `MembershipRevocationAlertDeliveryMetrics`
  - `attempted`
  - `succeeded`
  - `failed`
  - `deferred`
  - `buffered`
  - `dropped_capacity`
  - `dropped_retry_limit`
  - `dead_lettered`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：死信归档模型/存储抽象与实现完成。
  - **MR3**：恢复执行入口接入死信归档并补齐投递指标。
  - **MR4**：协同执行入口接入、测试完善、总文档与状态同步完成。
- Technical Risks:
  - 死信归档在极端故障下可能快速增长，需要后续引入 TTL 或归档压缩。
  - 文件型死信归档在多进程并发 append 下仍有竞争风险。
  - 指标为流程内聚合值，未接入外部监控时仍需主动拉取/日志采集。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-012-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-012-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
