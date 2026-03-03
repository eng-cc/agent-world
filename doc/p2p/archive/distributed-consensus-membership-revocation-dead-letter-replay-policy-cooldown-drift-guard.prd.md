> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略冷却窗口与漂移抑制

## 1. Executive Summary
- Problem Statement: 在现有策略自适应入口上增加**冷却窗口**，避免短时间内频繁上下调导致参数振荡。
- Proposed Solution: 增加**漂移抑制**，限制每轮策略调整幅度，避免 backlog 突变时策略阶跃过大。
- Success Criteria:
  - SC-1: 保持现有推荐与调度接口兼容，新增 guard 版本入口供渐进接入。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略冷却窗口与漂移抑制 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增带 guard 的策略推荐入口（自适应推荐 + 冷却窗口 + 单轮最大变更幅度）。
  - AC-2: 新增带 guard 的协调调度入口（推荐后执行，返回最终采用策略）。
  - AC-3: 增加 guard 参数校验与单元测试覆盖：
  - AC-4: 冷却窗口命中时保持当前策略；
  - AC-5: 漂移抑制命中时按步长截断策略变更；
  - AC-6: 协同调度联动 guard 策略执行。
- Non-Goals:
  - 基于全局监控系统的动态阈值自动学习。
  - 多节点间 adaptive policy 的强一致复制。
  - 跨 world 的统一策略编排与集中式配置中心。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：guard 推荐入口与参数校验完成。
  - **MR3**：guard 协同调度入口与单元测试完成。
  - **MR4**：总文档、项目状态、开发日志同步完成。
- Technical Risks:
  - 冷却窗口配置过大可能降低策略响应速度。
  - 步长限制过小可能导致高积压恢复过慢。
  - 若 guard 规则与基础推荐规则冲突，需通过测试固定优先级（先推荐再 guard）。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-018-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-018-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
