# Agent World Runtime：成员目录吊销死信优先级回放与跨节点回放协同（设计文档）

## 目标
- 为死信回放提供稳定且可解释的优先级策略，优先恢复高价值告警（重试上限触发）并兼顾历史积压。
- 为死信回放调度增加跨节点协同能力，避免多节点并发回放同一目标队列导致重复竞争。
- 保持既有回放与指标导出入口兼容，采用增量接口扩展。

## 范围

### In Scope（本次实现）
- 在 `replay_revocation_dead_letters(...)` 中引入优先级选择策略：
  - `RetryLimitExceeded` 高于 `CapacityEvicted`
  - 同优先级按 `attempt` 降序
  - 再按 `dropped_at_ms` 升序（老记录优先）
- 新增 `run_revocation_dead_letter_replay_schedule_coordinated(...)`：
  - 通过 `MembershipRevocationScheduleCoordinator` 获取回放执行 lease
  - lease key 维度为 `world_id + target_node_id`，支持跨节点协同
  - 成功获取 lease 后再执行现有调度回放逻辑
- 补充单元测试覆盖优先级顺序与协同互斥行为。

### Out of Scope（本次不做）
- 跨节点共享的 dead-letter 调度状态持久化（如统一 `last_replay_at_ms` store）。
- 自定义优先级权重配置中心或动态热更新。
- dead-letter 回放后的再次去重与幂等签名扩展。

## 接口 / 数据

### 回放优先级规则
- 输入：dead-letter 记录（`reason/attempt/dropped_at_ms`）。
- 规则：
  1) `reason`：`RetryLimitExceeded` > `CapacityEvicted`
  2) `attempt`：更高重试次数优先
  3) `dropped_at_ms`：更早丢弃优先
- 输出：按优先级截取 `max_replay` 条回放，其余记录回写 store。

### 协同调度入口
- 新增 `run_revocation_dead_letter_replay_schedule_coordinated(...)`
  - 参数新增：`coordinator_node_id`、`coordinator`、`coordinator_lease_ttl_ms`
  - 返回：`usize`（本次实际回放条数）
  - 语义：
    - 未获取 lease：返回 `0`
    - 获取 lease 且到期触发：返回实际回放数量
    - 保证 release 在执行后被调用

### 协同 key 规则
- 协同 world key：`{world_id}::revocation-dead-letter-replay::{target_node_id}`
- 通过该 key 保证同一 `world + target node` 在 lease 周期内仅有一个调度执行者。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：回放优先级策略编码完成并覆盖单测。
- **MR3**：跨节点协同调度入口完成并覆盖单测。
- **MR4**：总文档、项目状态、开发日志与验证命令更新完成。

## 风险
- 优先级规则固定可能导致低优先级死信长期滞后，需要后续引入配额或轮转策略。
- 协同仅依赖 lease，若 `last_replay_at_ms` 非共享，跨节点切换时可能出现“空转检查”。
- 复用现有协调器 key 需要严格规范字符串拼接，避免冲突或误共享。
