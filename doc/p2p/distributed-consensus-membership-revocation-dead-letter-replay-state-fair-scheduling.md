# Agent World Runtime：成员目录吊销死信回放状态持久化与公平调度（设计文档）

## 目标
- 为 dead-letter 回放调度增加可持久化状态，支持跨进程/跨节点重启后延续回放节奏。
- 在现有“高优先级优先”的基础上增加公平调度，降低低优先级记录长期饥饿风险。
- 保持现有回放入口兼容，采用增量接口扩展与可选接入方式。

## 范围

### In Scope（本次实现）
- 新增 dead-letter 回放状态模型：
  - `last_replay_at_ms`
  - `prefer_capacity_evicted`（公平调度提示位）
- 新增回放状态存储抽象与实现：
  - `MembershipRevocationDeadLetterReplayStateStore`
  - `InMemoryMembershipRevocationDeadLetterReplayStateStore`
  - `FileMembershipRevocationDeadLetterReplayStateStore`
- 新增公平回放策略：
  - `MembershipRevocationDeadLetterReplayPolicy`
  - 在高优先级优先前提下，限制连续高优先级回放条数
  - 有低优先级积压时按策略插入低优先级记录
- 新增状态持久化调度入口：
  - `run_revocation_dead_letter_replay_schedule_with_state_store(...)`
  - `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(...)`
- 补充单元测试覆盖公平回放、状态持久化、协同调度链路。

### Out of Scope（本次不做）
- 多维优先级（告警 code、world SLA、节点权重）动态配置。
- 分布式事务级别的强一致状态写入（例如共识写前日志）。
- 回放状态与投递指标的统一查询 API 聚合。

## 接口 / 数据

### 回放状态
- `MembershipRevocationDeadLetterReplayScheduleState`
  - `last_replay_at_ms: Option<i64>`
  - `prefer_capacity_evicted: bool`

### 回放策略
- `MembershipRevocationDeadLetterReplayPolicy`
  - `max_replay_per_run: usize`
  - `max_retry_limit_exceeded_streak: usize`

### 新增调度入口
- `run_revocation_dead_letter_replay_schedule_with_state_store(...)`
  - load state → interval 判定 → 公平回放 → save state
- `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(...)`
  - 先协调器 acquire，再执行 state-store 调度，再 release

### 公平调度原则
- 保持 `RetryLimitExceeded` 的更高处理优先级。
- 当 `CapacityEvicted` 存在积压时，不允许高优先级无限连续占满回放窗口。
- 使用 `prefer_capacity_evicted` 在多次调度间传递公平提示，减少重启后偏置。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：状态存储抽象与内存/文件实现完成。
- **MR3**：公平调度策略与 state-store 调度入口完成。
- **MR4**：测试、总文档、开发日志和项目状态同步完成。

## 风险
- 公平策略过于激进可能降低高优先级恢复速度，需要后续结合运行指标调参。
- 文件状态存储在异常退出时可能存在“上次回放时间”滞后，导致一次额外尝试。
- 协同锁与状态存储分离时，仍可能出现短暂空转（不会导致重复写入破坏）。
