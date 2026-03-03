> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销异常告警与对账调度自动化

## 1. Executive Summary
- Problem Statement: 为成员目录吊销对账结果提供标准化异常告警结构，降低人工巡检成本。
- Proposed Solution: 提供可复用的对账调度状态机，支持按时间间隔自动执行 checkpoint 发布与对账收敛。
- Success Criteria:
  - SC-1: 保持与现有 `membership.reconcile` 通道和 `MembershipRevocationReconcilePolicy` 兼容。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销异常告警与对账调度自动化 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增异常告警策略与告警数据结构：
  - AC-2: `MembershipRevocationAlertPolicy`
  - AC-3: `MembershipRevocationAlertSeverity`
  - AC-4: `MembershipRevocationAnomalyAlert`
  - AC-5: 新增告警评估接口：
  - AC-6: `evaluate_revocation_reconcile_alerts(...)`
- Non-Goals:
  - 告警消息推送到外部系统（邮件、Webhook、IM）。
  - 调度状态持久化到数据库或分布式 KV。
  - 对账调度任务的跨进程 leader 选举。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-alerting-scheduler.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-alerting-scheduler.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 异常告警
- `MembershipRevocationAlertPolicy`
  - `warn_diverged_threshold`
  - `critical_rejected_threshold`
- `MembershipRevocationAnomalyAlert`
  - `world_id`
  - `node_id`
  - `detected_at_ms`
  - `severity`
  - `code/message`
  - `drained/diverged/rejected`

### 调度自动化
- `MembershipRevocationReconcileSchedulePolicy`
  - `checkpoint_interval_ms`
  - `reconcile_interval_ms`
- `MembershipRevocationReconcileScheduleState`
  - `last_checkpoint_at_ms`
  - `last_reconcile_at_ms`
- `run_revocation_reconcile_schedule(...)`
  - 基于 interval 判定“是否到期”
  - 到期则执行 checkpoint 发布或对账收敛
  - 返回单轮执行报告，便于上层编排和可观测

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计/项目文档完成。
  - **MR2**：异常告警策略与评估接口实现。
  - **MR3**：对账调度策略/状态/执行入口实现。
  - **MR4**：单测、导出接口、总文档与日志更新。
- Technical Risks:
  - 阈值配置过低会造成告警噪音，过高会延迟异常发现。
  - 仅基于本地时钟调度，跨节点时钟漂移会影响执行节奏。
  - 未持久化调度状态时，进程重启后会触发一次“首次执行”行为。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-009-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-009-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
