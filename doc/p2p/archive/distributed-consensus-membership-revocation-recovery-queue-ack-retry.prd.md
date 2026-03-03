> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销恢复队列容量治理与告警 ACK 重试

## 1. Executive Summary
- Problem Statement: 为成员目录吊销告警恢复队列引入容量治理，避免下游长期不可用导致无界积压。
- Proposed Solution: 为告警恢复链路引入 ACK 重试策略（重试次数与重试退避），提高短时故障下的投递成功率。
- Success Criteria:
  - SC-1: 在保持现有恢复/协同接口兼容的前提下，增加可观测报告字段，支撑运维诊断。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销恢复队列容量治理与告警 ACK 重试 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增恢复队列元素结构，持久化 `attempt`、`next_retry_at_ms` 等 ACK 重试元数据。
  - AC-2: 新增 ACK 重试策略：
  - AC-3: `max_pending_alerts`
  - AC-4: `max_retry_attempts`
  - AC-5: `retry_backoff_ms`
  - AC-6: 新增带 ACK 重试的发送入口：
- Non-Goals:
  - 多级退避（指数退避/jitter）与按告警等级差异化策略。
  - 告警投递死信队列（DLQ）归档与专用检索接口。
  - 外部消息系统（Kafka/NATS）投递 ACK 协议适配。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-recovery-queue-ack-retry.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-recovery-queue-ack-retry.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 恢复队列元素
- `MembershipRevocationPendingAlert`
  - `alert: MembershipRevocationAnomalyAlert`
  - `attempt: usize`
  - `next_retry_at_ms: i64`
  - `last_error: Option<String>`

### ACK 重试策略
- `MembershipRevocationAlertAckRetryPolicy`
  - `max_pending_alerts`: 队列最大容量，超限触发容量治理。
  - `max_retry_attempts`: 单告警最大 ACK 失败重试次数。
  - `retry_backoff_ms`: ACK 失败后的最短重试等待时间。

### 恢复发送与协同编排
- `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry(...)`
  - 读取 pending 队列。
  - 对到期条目执行 ACK 投递；失败则更新 attempt/next_retry。
  - 新告警尝试即时发送；失败入队。
  - 按 `max_pending_alerts` 执行容量裁剪并回写。
- `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry(...)`
  - 与既有协调器 + schedule + dedup 流程集成。
  - 在告警发射阶段使用 ACK 重试策略。

### 报告字段扩展
- `MembershipRevocationAlertRecoveryReport`
  - `recovered`
  - `emitted_new`
  - `buffered`
  - `deferred`
  - `dropped_capacity`
  - `dropped_retry_limit`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计/项目文档完成。
  - **MR2**：恢复队列结构与存储格式升级完成（含兼容读取）。
  - **MR3**：ACK 重试 + 容量治理发送入口完成。
  - **MR4**：协同编排入口集成、单元测试与总文档更新完成。
- Technical Risks:
  - 队列容量治理策略若配置过小，可能在持续故障时丢弃过多新告警。
  - 统一退避参数在异构下游通道下可能不够精细，存在恢复速度与压力折中。
  - 旧格式恢复文件兼容读取若异常，需要明确回退为安全空队列并记录错误。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-033-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-033-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
