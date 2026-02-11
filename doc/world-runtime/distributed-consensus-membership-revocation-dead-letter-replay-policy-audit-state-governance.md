# Agent World Runtime：成员目录吊销死信回放策略审计状态持久化与多级回退治理（设计文档）

## 目标
- 为回滚异常告警补齐状态存储能力，确保节点重启后仍能保持告警冷却语义。
- 在策略回滚连续触发场景下引入多级回退治理，避免策略在异常窗口内反复振荡。
- 保持现有“持久化推荐 + 审计 + 告警”链路兼容，新增“状态存储 + 治理”增强入口。

## 范围

### In Scope（本次实现）
- 新增回滚告警状态存储抽象（内存/文件实现）。
- 新增回滚治理状态模型、治理策略与状态存储抽象（内存/文件实现）。
- 新增多级回退治理入口：在现有审计/告警执行后，对连续回滚进行 level 化治理。
- 新增单元测试：状态存储 round-trip、治理升级、冷却跨运行保持、参数校验。

### Out of Scope（本次不做）
- 跨节点共享治理状态与统一仲裁。
- 治理策略与业务指标的自动学习调参。
- 外部控制面统一下发治理级别。

## 接口 / 数据

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

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：回滚告警状态存储与治理状态存储实现完成。
- **MR3**：多级回退治理联动入口与测试完成。
- **MR4**：总文档、项目状态、开发日志同步完成。

## 风险
- 治理阈值设置过低会过早进入 emergency 策略，影响回放吞吐。
- 治理级别切换策略过于激进可能导致稳定策略长期无法恢复。
- 文件状态损坏会影响治理连续性，需要保持默认值可回退。
