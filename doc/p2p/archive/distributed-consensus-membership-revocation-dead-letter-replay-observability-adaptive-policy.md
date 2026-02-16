> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放状态观测聚合与策略自适应（设计文档）

## 目标
- 在 dead-letter 回放链路中提供统一观测聚合入口，整合 replay state、dead-letter backlog、pending 队列和投递指标。
- 提供可解释的策略自适应能力，自动给出下一轮 `MembershipRevocationDeadLetterReplayPolicy` 建议值。
- 保持现有调度入口兼容，支持“仅推荐”和“推荐+执行”两种使用模式。

## 范围

### In Scope（本次实现）
- 新增策略推荐入口 `recommend_revocation_dead_letter_replay_policy(...)`。
- 聚合数据来源：
  - `MembershipRevocationDeadLetterReplayStateStore`（last replay + prefer flag）
  - `MembershipRevocationAlertDeadLetterStore`（dead-letter 与 delivery metrics）
  - `MembershipRevocationAlertRecoveryStore`（pending 队列）
- 新增编排入口 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_adaptive_policy(...)`。
- 补充单元测试覆盖高积压扩容、低积压收敛、公平倾斜与编排联动。

### Out of Scope（本次不做）
- 基于外部监控系统（Prometheus/OTel）的自动反馈闭环。
- 自适应策略的多节点强一致同步。
- 机器学习策略或历史长期趋势预测。

## 接口 / 数据

### 策略推荐
- `recommend_revocation_dead_letter_replay_policy(...)`
  - 输入：当前 policy + 观测窗口参数（metrics lookback、策略上下界）
  - 输出：建议 policy（仅内存返回，不自动持久化）

### 推荐后执行
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_adaptive_policy(...)`
  - 先计算建议 policy
  - 再执行协调 + state-store 调度
  - 返回 `(replayed, recommended_policy)`

### 自适应规则（实现级）
- backlog 高压：提高 `max_replay_per_run`。
- backlog 低压且近期健康：降低 `max_replay_per_run`。
- 存在容量型积压且公平提示位命中：降低 `max_retry_limit_exceeded_streak`，提升低优先级可达性。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：观测聚合 + 推荐策略入口完成。
- **MR3**：推荐后执行编排入口完成。
- **MR4**：测试、总文档、开发日志与项目状态同步完成。

## 风险
- 启发式规则对不同负载敏感，阈值可能需要在线调优。
- 观测窗口过短会导致策略抖动，过长会导致响应迟钝。
- 自动推荐若被直接用于生产执行，需要配合灰度和上限保护。
