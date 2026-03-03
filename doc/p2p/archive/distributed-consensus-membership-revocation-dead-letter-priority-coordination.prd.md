> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信优先级回放与跨节点回放协同

## 1. Executive Summary
- Problem Statement: 为死信回放提供稳定且可解释的优先级策略，优先恢复高价值告警（重试上限触发）并兼顾历史积压。
- Proposed Solution: 为死信回放调度增加跨节点协同能力，避免多节点并发回放同一目标队列导致重复竞争。
- Success Criteria:
  - SC-1: 保持既有回放与指标导出入口兼容，采用增量接口扩展。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信优先级回放与跨节点回放协同 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `replay_revocation_dead_letters(...)` 中引入优先级选择策略：
  - AC-2: `RetryLimitExceeded` 高于 `CapacityEvicted`
  - AC-3: 同优先级按 `attempt` 降序
  - AC-4: 再按 `dropped_at_ms` 升序（老记录优先）
  - AC-5: 新增 `run_revocation_dead_letter_replay_schedule_coordinated(...)`：
  - AC-6: 通过 `MembershipRevocationScheduleCoordinator` 获取回放执行 lease
- Non-Goals:
  - 跨节点共享的 dead-letter 调度状态持久化（如统一 `last_replay_at_ms` store）。
  - 自定义优先级权重配置中心或动态热更新。
  - dead-letter 回放后的再次去重与幂等签名扩展。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-priority-coordination.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-priority-coordination.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：回放优先级策略编码完成并覆盖单测。
  - **MR3**：跨节点协同调度入口完成并覆盖单测。
  - **MR4**：总文档、项目状态、开发日志与验证命令更新完成。
- Technical Risks:
  - 优先级规则固定可能导致低优先级死信长期滞后，需要后续引入配额或轮转策略。
  - 协同仅依赖 lease，若 `last_replay_at_ms` 非共享，跨节点切换时可能出现“空转检查”。
  - 复用现有协调器 key 需要严格规范字符串拼接，避免冲突或误共享。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-013-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-013-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
