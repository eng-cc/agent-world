> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理审计归档分层转储与演练告警联动

## 1. Executive Summary
- Problem Statement: 为治理审计归档增加热/冷分层能力，避免热归档无限增长并保留近期可快速查询窗口。
- Proposed Solution: 提供可重入的分层转储入口，在失败场景下提供补偿回滚，避免冷热层数据失配。
- Success Criteria:
  - SC-1: 在恢复演练调度链路增加异常告警联动，实现“演练执行→异常判定→告警落盘”闭环。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理审计归档分层转储与演练告警联动 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增治理审计归档分层转储策略模型（热层保留、转储年龄门槛、单次转储上限）。
  - AC-2: 新增分层转储执行入口与运行报告，支持热/冷归档存储协同。
  - AC-3: 新增转储失败补偿机制（热层写失败时回滚冷层写入）。
  - AC-4: 新增恢复演练异常告警策略、告警状态存储（内存/文件）与告警执行入口。
  - AC-5: 新增“归档保留+分层转储+演练调度+告警联动”编排入口。
  - AC-6: 补充单元测试：分层转储、补偿回滚、告警冷却、联动编排。
- Non-Goals:
  - 冷层对象存储（S3/OSS）适配与远程生命周期管理。
  - 多节点统一冷热层聚合查询。
  - 告警通知外送（Webhook/IM）与事件总线。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 分层转储策略（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadPolicy`
  - `hot_max_records`
  - `offload_min_age_ms`
  - `max_offload_records`

### 分层转储报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadReport`
  - `world_id/node_id/offloaded_at_ms`
  - `hot_before/hot_after`
  - `cold_before/cold_after`
  - `offloaded`
  - `offloaded_by_age/offloaded_by_capacity`
  - `kept_due_to_rate_limit`

### 演练告警策略与状态（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertPolicy`
  - `max_alert_silence_ms`
  - `rollback_streak_threshold`
  - `alert_cooldown_ms`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertState`
  - `last_alert_at_ms`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore`
  - `load/save`（内存/文件实现）

### 联动编排报告（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertRunReport`
  - `prune_report`
  - `offload_report`
  - `drill_run_report`
  - `drill_alert_report`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：分层转储策略、执行入口与补偿机制实现完成。
  - **MR3**：演练异常告警策略/状态存储/告警入口实现完成。
  - **MR4**：联动编排入口、测试、总文档与 devlog 同步完成。
- Technical Risks:
  - 分层阈值设置不当可能导致热层抖动或冷层增长过快。
  - 补偿回滚仅覆盖冷热层写顺序失配，仍需后续补齐跨进程幂等保障。
  - 告警判定规则过严会产生噪声，过松会漏报，需要后续基于线上指标调优。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-022-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-022-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
