> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略建议持久化与回滚保护

## 1. Executive Summary
- Problem Statement: 为 dead-letter 回放策略引入持久化存储，避免节点重启后丢失最近推荐与稳定策略。
- Proposed Solution: 为策略推荐链路增加回滚保护，当近期投递指标恶化时自动回退到最近稳定策略。
- Success Criteria:
  - SC-1: 保持现有自适应推荐与调度入口兼容，新增“持久化推荐+执行”入口供渐进接入。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略建议持久化与回滚保护 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增策略状态模型与策略状态存储抽象（内存/文件实现）。
  - AC-2: 新增带持久化状态的策略推荐入口，支持稳定策略更新与回滚判定。
  - AC-3: 新增带持久化推荐的协同调度入口，返回回放数量、采用策略与是否触发回滚。
  - AC-4: 补充单元测试：状态存储 round-trip、回滚命中、稳定策略推进、联动调度。
- Non-Goals:
  - 策略状态跨节点强一致同步。
  - 外部控制面统一下发策略状态。
  - 基于机器学习的自动阈值调整。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：策略状态存储与回滚 guard 实现完成。
  - **MR3**：持久化推荐与协同调度入口完成。
  - **MR4**：测试、总文档、开发日志与项目状态同步完成。
- Technical Risks:
  - 回滚阈值配置过敏会导致策略频繁回退，影响吞吐。
  - 稳定策略判定过宽会放大“坏策略”存留时间。
  - 文件状态损坏会影响推荐入口，需要保持默认值可恢复。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-030-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-030-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
