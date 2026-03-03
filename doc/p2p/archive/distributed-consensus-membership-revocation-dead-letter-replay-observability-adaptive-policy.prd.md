> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放状态观测聚合与策略自适应

## 1. Executive Summary
- Problem Statement: 在 dead-letter 回放链路中提供统一观测聚合入口，整合 replay state、dead-letter backlog、pending 队列和投递指标。
- Proposed Solution: 提供可解释的策略自适应能力，自动给出下一轮 `MembershipRevocationDeadLetterReplayPolicy` 建议值。
- Success Criteria:
  - SC-1: 保持现有调度入口兼容，支持“仅推荐”和“推荐+执行”两种使用模式。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放状态观测聚合与策略自适应 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增策略推荐入口 `recommend_revocation_dead_letter_replay_policy(...)`。
  - AC-2: 聚合数据来源：
  - AC-3: `MembershipRevocationDeadLetterReplayStateStore`（last replay + prefer flag）
  - AC-4: `MembershipRevocationAlertDeadLetterStore`（dead-letter 与 delivery metrics）
  - AC-5: `MembershipRevocationAlertRecoveryStore`（pending 队列）
  - AC-6: 新增编排入口 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_adaptive_policy(...)`。
- Non-Goals:
  - 基于外部监控系统（Prometheus/OTel）的自动反馈闭环。
  - 自适应策略的多节点强一致同步。
  - 机器学习策略或历史长期趋势预测。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：观测聚合 + 推荐策略入口完成。
  - **MR3**：推荐后执行编排入口完成。
  - **MR4**：测试、总文档、开发日志与项目状态同步完成。
- Technical Risks:
  - 启发式规则对不同负载敏感，阈值可能需要在线调优。
  - 观测窗口过短会导致策略抖动，过长会导致响应迟钝。
  - 自动推荐若被直接用于生产执行，需要配合灰度和上限保护。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-015-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-015-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
