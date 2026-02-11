# Agent World Runtime：成员目录吊销死信回放策略冷却窗口与漂移抑制（设计文档）

## 目标
- 在现有策略自适应入口上增加**冷却窗口**，避免短时间内频繁上下调导致参数振荡。
- 增加**漂移抑制**，限制每轮策略调整幅度，避免 backlog 突变时策略阶跃过大。
- 保持现有推荐与调度接口兼容，新增 guard 版本入口供渐进接入。

## 范围

### In Scope（本次实现）
- 新增带 guard 的策略推荐入口（自适应推荐 + 冷却窗口 + 单轮最大变更幅度）。
- 新增带 guard 的协调调度入口（推荐后执行，返回最终采用策略）。
- 增加 guard 参数校验与单元测试覆盖：
  - 冷却窗口命中时保持当前策略；
  - 漂移抑制命中时按步长截断策略变更；
  - 协同调度联动 guard 策略执行。

### Out of Scope（本次不做）
- 基于全局监控系统的动态阈值自动学习。
- 多节点间 adaptive policy 的强一致复制。
- 跨 world 的统一策略编排与集中式配置中心。

## 接口 / 数据

### guard 推荐入口（拟）
- `recommend_revocation_dead_letter_replay_policy_with_adaptive_guard(...)`
  - 输入：`now_ms`、当前 policy、观测窗口、策略上下界、guard 参数。
  - 输出：经过冷却与漂移抑制后的推荐 policy。

### guard 协同调度入口（拟）
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_guarded_adaptive_policy(...)`
  - 流程：`recommend(adaptive)` → `guard(cooldown + drift clamp)` → `coordinated replay run`
  - 返回：`(replayed, guarded_policy)`

### guard 参数（拟）
- `policy_cooldown_ms`：策略调整最小间隔。
- `max_replay_step_change`：单轮 `max_replay_per_run` 最大变化量。
- `max_retry_streak_step_change`：单轮 `max_retry_limit_exceeded_streak` 最大变化量。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：guard 推荐入口与参数校验完成。
- **MR3**：guard 协同调度入口与单元测试完成。
- **MR4**：总文档、项目状态、开发日志同步完成。

## 风险
- 冷却窗口配置过大可能降低策略响应速度。
- 步长限制过小可能导致高积压恢复过慢。
- 若 guard 规则与基础推荐规则冲突，需通过测试固定优先级（先推荐再 guard）。
