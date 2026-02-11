# Agent World Runtime：分布式计算与存储（项目管理文档）

## 文档拆分说明
- 主文档保留活跃里程碑（P3.25+）与当前状态，便于持续推进。
- 历史里程碑（0~41）拆分到：`doc/world-runtime/distributed-runtime.project.archive-0-41.md`。

## 任务拆解（活跃分卷）
### 42. P3.25 成员目录吊销死信优先级回放与跨节点回放协同
- [x] `replay_revocation_dead_letters` 按 reason/attempt/dropped_at 优先级回放
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated`
- [x] 回放协同 lease key 按 `world_id + target_node_id` 维度隔离
- [x] 新增优先级回放与跨节点协同单元测试
- [x] 单元测试与分布式回归

### 43. P3.26 成员目录吊销死信回放状态持久化与公平调度
- [x] 新增 `MembershipRevocationDeadLetterReplayStateStore` 与内存/文件实现
- [x] 新增 `MembershipRevocationDeadLetterReplayScheduleState`
- [x] 新增 `MembershipRevocationDeadLetterReplayPolicy`（公平调度参数）
- [x] 新增 `replay_revocation_dead_letters_with_policy`
- [x] 新增 `run_revocation_dead_letter_replay_schedule_with_state_store`
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store`
- [x] 新增状态持久化与公平调度单元测试
- [x] 单元测试与分布式回归

### 44. P3.27 成员目录吊销死信回放状态观测聚合与策略自适应
- [x] 新增 `recommend_revocation_dead_letter_replay_policy`
- [x] 新增 `validate_adaptive_policy_bounds` 与 delivery metrics 聚合工具函数
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_adaptive_policy`
- [x] 新增策略自适应单元测试（扩容、收敛、公平倾斜、联动执行）
- [x] 单元测试与分布式回归

### 45. P3.28 成员目录吊销死信回放策略冷却窗口与漂移抑制
- [x] 新增 `recommend_revocation_dead_letter_replay_policy_with_adaptive_guard`
- [x] 新增 `validate_adaptive_policy_guard_bounds` 与策略步长截断 helper
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_guarded_adaptive_policy`
- [x] 新增冷却窗口与漂移抑制单元测试
- [x] 单元测试与分布式回归

### 46. P3.29 成员目录吊销死信回放策略建议持久化与回滚保护
- [x] 新增 `MembershipRevocationDeadLetterReplayPolicyStore` 与内存/文件实现
- [x] 新增 `MembershipRevocationDeadLetterReplayPolicyState`
- [x] 新增 `MembershipRevocationDeadLetterReplayRollbackGuard`
- [x] 新增 `recommend_revocation_dead_letter_replay_policy_with_persistence_and_rollback_guard`
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy`
- [x] 新增持久化/回滚保护单元测试
- [x] 单元测试与分布式回归

### 47. P3.30 成员目录吊销死信回放策略采纳审计与异常回退告警
- [x] 新增 `MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord/Decision`
- [x] 新增 `MembershipRevocationDeadLetterReplayPolicyAuditStore` 与内存/文件实现
- [x] 新增 `MembershipRevocationDeadLetterReplayRollbackAlertPolicy/State`
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_and_alert`
- [x] 新增策略采纳审计与回退告警单元测试
- [x] 单元测试与分布式回归

### 48. P3.31 成员目录吊销死信回放策略审计状态持久化与多级回退治理
- [x] 新增 `MembershipRevocationDeadLetterReplayRollbackAlertStateStore` 与内存/文件实现
- [x] 新增 `MembershipRevocationDeadLetterReplayRollbackGovernancePolicy/State` 与内存/文件状态存储
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_and_governance`
- [x] 新增治理级别（`Normal/Stable/Emergency`）与策略覆盖逻辑
- [x] 新增状态持久化/治理升级单元测试
- [x] 单元测试与分布式回归

### 49. P3.32 成员目录吊销死信回放策略治理审计归档与恢复演练
- [x] 新增 `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord`
- [x] 新增 `MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore` 与内存/文件实现
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_governance_and_archive`
- [x] 新增 `run_revocation_dead_letter_replay_rollback_governance_recovery_drill`
- [x] 新增治理归档与恢复演练单元测试
- [x] 单元测试与分布式回归

### 50. P3.33 成员目录吊销死信回放策略治理审计归档保留策略与演练调度
- [x] 新增治理审计归档保留策略（按条数/时间窗口）
- [x] 新增归档裁剪执行入口与文件归档裁剪实现
- [x] 新增恢复演练调度策略与状态存储（内存/文件）
- [x] 新增演练调度编排入口与运行报告
- [x] 补充归档保留与演练调度单元测试
- [x] 单元测试与分布式回归

### 51. P3.34 成员目录吊销死信回放策略治理审计归档分层转储与演练告警联动
- [x] 新增归档分层转储策略（热/冷分层）
- [x] 新增转储执行入口与失败补偿机制
- [x] 新增演练结果异常告警策略与告警落盘
- [x] 新增转储+演练+告警联动编排入口
- [x] 补充分层转储与告警联动单元测试
- [x] 单元测试与分布式回归

### 52. P3.35 成员目录吊销死信回放策略治理审计聚合查询与演练告警事件总线
- [x] 新增治理审计 hot/cold 聚合查询策略与查询入口
- [x] 新增聚合查询报告与参数校验
- [x] 新增恢复演练告警事件模型与事件总线（内存/文件）
- [x] 新增转储+演练+告警+事件总线联动入口
- [x] 补充聚合查询与事件总线联动单元测试
- [x] 单元测试与分布式回归

### 53. P3.36 成员目录吊销死信回放策略治理恢复演练告警事件总线聚合拉取与分页增量查询
- [x] 新增事件总线跨节点聚合拉取入口
- [x] 支持按最小时间窗口过滤
- [x] 支持按演练告警 outcome 过滤
- [x] 支持 offset + max_records 分页增量查询
- [x] 补充聚合拉取/分页/参数校验单元测试
- [x] 单元测试与分布式回归

### 54. P3.37 成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位拉取与 outcome 汇总
- [x] 新增按时间水位的跨节点增量拉取入口（严格 `>` since）
- [x] 新增按 outcome 的聚合汇总入口
- [x] 增量拉取返回顺序调整为时间正序（便于消费端顺序处理）
- [x] 补充增量水位与 outcome 汇总单元测试
- [x] 单元测试与分布式回归

### 55. P3.38 成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位自动推进与空拉取幂等
- [x] 新增增量拉取 + 下一水位联动查询入口
- [x] 保证水位单调不回退（空拉取保持原水位）
- [x] 支持与既有 outcome 过滤和 `max_records` 限流组合
- [x] 补充分批拉取/空拉取幂等/水位推进单元测试
- [x] 单元测试与分布式回归

### 56. P3.39 成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标增量续拉
- [x] 新增时间水位 + 节点游标的复合续拉查询入口
- [x] 支持同毫秒跨节点事件的稳定续拉
- [x] 支持 cursor 空拉取幂等保持
- [x] 补充同毫秒多节点场景单元测试
- [x] 单元测试与分布式回归

### 57. P3.40 成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标序号续拉
- [x] 新增时间水位 + 节点 + 节点内序号复合续拉查询入口
- [x] 支持同节点同毫秒多事件稳定续拉
- [x] 支持复合游标空拉取幂等保持
- [x] 补充同节点同毫秒分页续拉单元测试
- [x] 单元测试与分布式回归

## 依赖
- `doc/world-runtime.md`
- `doc/world-runtime/runtime-integration.md`
- `doc/world-runtime/module-storage.md`
- libp2p 协议栈与实现

## 状态
- 当前阶段：P3.40 完成（成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标序号续拉）
- 下一步：P3.41（待规划）
- 最近更新：成员目录吊销死信回放策略治理恢复演练告警事件总线复合游标序号续拉（2026-02-11）
