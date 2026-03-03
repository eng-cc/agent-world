> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销告警上报与调度状态持久化

## 1. Executive Summary
- Problem Statement: 将吊销对账异常告警从“内存结果”升级为“可上报 sink”，支持后续接入外部告警系统。
- Proposed Solution: 为对账调度状态提供统一 store 抽象，实现跨重启恢复调度进度。
- Success Criteria:
  - SC-1: 保持与现有对账调度/告警评估接口兼容，最小改造上层调用。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销告警上报与调度状态持久化 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增告警上报抽象与实现：
  - AC-2: `MembershipRevocationAlertSink`
  - AC-3: `InMemoryMembershipRevocationAlertSink`
  - AC-4: `FileMembershipRevocationAlertSink`
  - AC-5: 新增调度状态存储抽象与实现：
  - AC-6: `MembershipRevocationScheduleStateStore`
- Non-Goals:
  - 对接外部告警服务（Webhook/IM/邮件）。
  - 多副本状态 store 的分布式一致性（锁/选主）。
  - 告警去重抑制、降噪策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-alert-delivery-state-store.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-alert-delivery-state-store.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 告警上报
- `MembershipRevocationAlertSink::emit(alert)`
- `InMemoryMembershipRevocationAlertSink::list()`
- `FileMembershipRevocationAlertSink::list(world_id)`

### 调度状态持久化
- `MembershipRevocationScheduleStateStore::load(world_id, node_id)`
- `MembershipRevocationScheduleStateStore::save(world_id, node_id, state)`
- `MembershipRevocationReconcileScheduleState`
  - `last_checkpoint_at_ms`
  - `last_reconcile_at_ms`

### 编排入口
- `run_revocation_reconcile_schedule_with_store_and_alerts(...)`
  - 从 store 读取状态
  - 执行 schedule（checkpoint/reconcile）
  - 写回最新状态
  - 评估并上报告警

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计/项目文档完成。
  - **MR2**：告警 sink 抽象与文件/内存实现完成。
  - **MR3**：调度状态 store 抽象与文件/内存实现完成。
  - **MR4**：编排入口、单测、导出与总文档更新完成。
- Technical Risks:
  - 本地文件存储在高并发写入场景可能存在竞争，需要上层串行化调度。
  - 仅按 world/node 维度持久化，若节点标识漂移会造成状态分裂。
  - 告警无去重策略时，持续异常会导致 JSONL 快速增长。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-008-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-008-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
