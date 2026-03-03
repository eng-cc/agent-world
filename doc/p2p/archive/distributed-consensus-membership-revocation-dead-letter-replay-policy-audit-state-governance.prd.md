> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略审计状态持久化与多级回退治理

## 1. Executive Summary
- Problem Statement: 为回滚异常告警补齐状态存储能力，确保节点重启后仍能保持告警冷却语义。
- Proposed Solution: 在策略回滚连续触发场景下引入多级回退治理，避免策略在异常窗口内反复振荡。
- Success Criteria:
  - SC-1: 保持现有“持久化推荐 + 审计 + 告警”链路兼容，新增“状态存储 + 治理”增强入口。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略审计状态持久化与多级回退治理 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增回滚告警状态存储抽象（内存/文件实现）。
  - AC-2: 新增回滚治理状态模型、治理策略与状态存储抽象（内存/文件实现）。
  - AC-3: 新增多级回退治理入口：在现有审计/告警执行后，对连续回滚进行 level 化治理。
  - AC-4: 新增单元测试：状态存储 round-trip、治理升级、冷却跨运行保持、参数校验。
- Non-Goals:
  - 跨节点共享治理状态与统一仲裁。
  - 治理策略与业务指标的自动学习调参。
  - 外部控制面统一下发治理级别。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 回滚告警状态存储（拟）
- `MembershipRevocationDeadLetterReplayRollbackAlertStateStore`
  - `load_alert_state(world_id, node_id)`
  - `save_alert_state(world_id, node_id, state)`
- 实现：`InMemory...RollbackAlertStateStore`、`File...RollbackAlertStateStore`。

### 多级回退治理（拟）
- `MembershipRevocationDeadLetterReplayRollbackGovernancePolicy`
  - `level_one_rollback_streak`
  - `level_two_rollback_streak`
  - `level_two_emergency_policy`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceLevel`
  - `Normal` / `Stable` / `Emergency`
- `MembershipRevocationDeadLetterReplayRollbackGovernanceState`
  - `rollback_streak`
  - `last_level`

### 联动入口（拟）
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_and_governance(...)`
  - 输入：现有审计告警参数 + alert_state_store + governance_policy/store
  - 输出：`(replayed, policy, rolled_back, alert_emitted, governance_level)`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：回滚告警状态存储与治理状态存储实现完成。
  - **MR3**：多级回退治理联动入口与测试完成。
  - **MR4**：总文档、项目状态、开发日志同步完成。
- Technical Risks:
  - 治理阈值设置过低会过早进入 emergency 策略，影响回放吞吐。
  - 治理级别切换策略过于激进可能导致稳定策略长期无法恢复。
  - 文件状态损坏会影响治理连续性，需要保持默认值可回退。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-017-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-017-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
