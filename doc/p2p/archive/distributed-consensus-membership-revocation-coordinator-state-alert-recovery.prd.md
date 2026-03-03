> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销协同状态外部存储与告警恢复机制

## 1. Executive Summary
- Problem Statement: 为成员目录吊销调度协同锁增加外部状态存储抽象，支持跨进程/跨重启延续协调状态。
- Proposed Solution: 在告警发送链路中增加“失败缓冲 + 恢复重放”机制，降低下游不可用时的告警丢失风险。
- Success Criteria:
  - SC-1: 与既有调度、去重、协同编排接口保持兼容并支持渐进接入。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销协同状态外部存储与告警恢复机制 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增协同状态存储抽象与实现：
  - AC-2: `MembershipRevocationCoordinatorStateStore`
  - AC-3: `InMemoryMembershipRevocationCoordinatorStateStore`
  - AC-4: `FileMembershipRevocationCoordinatorStateStore`
  - AC-5: `MembershipRevocationCoordinatorLeaseState`
  - AC-6: 新增基于 store 的协同器实现：
- Non-Goals:
  - 外部一致性存储（Redis/etcd）正式适配与高可用部署。
  - 告警 ACK/重试退避策略分级控制。
  - 协同状态和恢复队列加密存储与访问审计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 协同状态外部存储
- `MembershipRevocationCoordinatorStateStore`
  - `load(world_id)`
  - `save(world_id, lease_state)`
  - `clear(world_id)`
- `MembershipRevocationCoordinatorLeaseState`
  - `holder_node_id`
  - `expires_at_ms`

### 告警恢复机制
- `MembershipRevocationAlertRecoveryStore`
  - `load_pending(world_id, node_id)`
  - `save_pending(world_id, node_id, alerts)`
- `emit_revocation_reconcile_alerts_with_recovery(...)`
  - 先重放 pending，再发送新告警
  - 发送失败剩余告警回写 pending

### 协同恢复编排
- `run_revocation_reconcile_coordinated_with_recovery(...)`
  - 协调器抢锁
  - 读取并执行 schedule 状态
  - 评估/去重告警
  - 按恢复机制发送并缓存失败告警
  - 返回 `MembershipRevocationCoordinatedRecoveryRunReport`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计/项目文档完成。
  - **MR2**：协同状态外部存储抽象与实现完成。
  - **MR3**：告警恢复存储与恢复发送入口完成。
  - **MR4**：协同恢复编排、测试、导出与总文档更新完成。
- Technical Risks:
  - 文件存储在并发写入场景仍有竞争风险，生产需配合外部锁或串行调度。
  - 恢复队列未设置容量上限时，长期下游故障可能导致积压增长。
  - 节点时钟偏移会影响协同租约过期判断的准确性。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-011-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-011-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
