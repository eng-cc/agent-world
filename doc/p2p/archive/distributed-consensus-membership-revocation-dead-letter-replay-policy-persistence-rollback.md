> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略建议持久化与回滚保护（设计文档）

## 目标
- 为 dead-letter 回放策略引入持久化存储，避免节点重启后丢失最近推荐与稳定策略。
- 为策略推荐链路增加回滚保护，当近期投递指标恶化时自动回退到最近稳定策略。
- 保持现有自适应推荐与调度入口兼容，新增“持久化推荐+执行”入口供渐进接入。

## 范围

### In Scope（本次实现）
- 新增策略状态模型与策略状态存储抽象（内存/文件实现）。
- 新增带持久化状态的策略推荐入口，支持稳定策略更新与回滚判定。
- 新增带持久化推荐的协同调度入口，返回回放数量、采用策略与是否触发回滚。
- 补充单元测试：状态存储 round-trip、回滚命中、稳定策略推进、联动调度。

### Out of Scope（本次不做）
- 策略状态跨节点强一致同步。
- 外部控制面统一下发策略状态。
- 基于机器学习的自动阈值调整。

## 接口 / 数据

### 策略状态（拟）
- `MembershipRevocationDeadLetterReplayPolicyState`
  - `active_policy`：当前采用策略
  - `last_stable_policy`：最近稳定策略
  - `last_policy_update_at_ms`：最近策略更新时刻
  - `last_stable_at_ms`：最近稳定确认时刻
  - `last_rollback_at_ms`：最近回滚时刻

### 回滚保护参数（拟）
- `MembershipRevocationDeadLetterReplayRollbackGuard`
  - `min_attempted`：触发回滚判定所需最小投递样本量
  - `failure_ratio_per_mille`：失败比例阈值
  - `dead_letter_ratio_per_mille`：dead-letter 比例阈值
  - `rollback_cooldown_ms`：回滚冷却窗口

### 入口（拟）
- `recommend_revocation_dead_letter_replay_policy_with_persistence_and_rollback_guard(...)`
  - 输入：基础策略、adaptive 参数、guard 参数、策略状态 store
  - 输出：`(recommended_policy, rolled_back)`
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy(...)`
  - 推荐后执行协同回放，返回 `(replayed, policy, rolled_back)`。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：策略状态存储与回滚 guard 实现完成。
- **MR3**：持久化推荐与协同调度入口完成。
- **MR4**：测试、总文档、开发日志与项目状态同步完成。

## 风险
- 回滚阈值配置过敏会导致策略频繁回退，影响吞吐。
- 稳定策略判定过宽会放大“坏策略”存留时间。
- 文件状态损坏会影响推荐入口，需要保持默认值可恢复。
