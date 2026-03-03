> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放状态持久化与公平调度

## 1. Executive Summary
- Problem Statement: 为 dead-letter 回放调度增加可持久化状态，支持跨进程/跨节点重启后延续回放节奏。
- Proposed Solution: 在现有“高优先级优先”的基础上增加公平调度，降低低优先级记录长期饥饿风险。
- Success Criteria:
  - SC-1: 保持现有回放入口兼容，采用增量接口扩展与可选接入方式。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放状态持久化与公平调度 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增 dead-letter 回放状态模型：
  - AC-2: `last_replay_at_ms`
  - AC-3: `prefer_capacity_evicted`（公平调度提示位）
  - AC-4: 新增回放状态存储抽象与实现：
  - AC-5: `MembershipRevocationDeadLetterReplayStateStore`
  - AC-6: `InMemoryMembershipRevocationDeadLetterReplayStateStore`
- Non-Goals:
  - 多维优先级（告警 code、world SLA、节点权重）动态配置。
  - 分布式事务级别的强一致状态写入（例如共识写前日志）。
  - 回放状态与投递指标的统一查询 API 聚合。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：状态存储抽象与内存/文件实现完成。
  - **MR3**：公平调度策略与 state-store 调度入口完成。
  - **MR4**：测试、总文档、开发日志和项目状态同步完成。
- Technical Risks:
  - 公平策略过于激进可能降低高优先级恢复速度，需要后续结合运行指标调参。
  - 文件状态存储在异常退出时可能存在“上次回放时间”滞后，导致一次额外尝试。
  - 协同锁与状态存储分离时，仍可能出现短暂空转（不会导致重复写入破坏）。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-031-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-031-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
