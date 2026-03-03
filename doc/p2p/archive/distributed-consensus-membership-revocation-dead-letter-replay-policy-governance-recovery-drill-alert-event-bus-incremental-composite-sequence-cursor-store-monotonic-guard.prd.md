> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合序号游标状态单调推进守卫（P3.42）

## 1. Executive Summary
- Problem Statement: 在 P3.41 游标状态持久化基础上，增加存储层单调推进校验，阻止游标被旧值覆盖回退。
- Proposed Solution: 将游标比较规则统一为复合键比较：`(since_event_at_ms, since_node_id, since_node_event_offset)`。
- Success Criteria:
  - SC-1: 保持已有一体化续拉接口不变，仅增强状态保存时的安全约束。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线复合序号游标状态单调推进守卫（P3.42） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **包含**：
  - AC-2: 新增复合游标比较辅助函数。
  - AC-3: 在内存/文件游标状态 store 中启用“禁止回退写入”校验。
  - AC-4: 新增单元测试覆盖：允许前进、允许幂等同值、拒绝回退。
  - AC-5: **不包含**：
  - AC-6: 多消费者分布式锁/租约
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store-monotonic-guard.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store-monotonic-guard.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口/数据
- 比较维度：
  - `since_event_at_ms`
  - `since_node_id`（`None` 视为最小值）
  - `since_node_event_offset`
- 存储行为：
  - 新值 > 旧值：允许
  - 新值 == 旧值：允许（幂等）
  - 新值 < 旧值：拒绝并返回 `DistributedValidationFailed`

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档与项目管理文档。
  - M2：完成游标比较与单调守卫实现。
  - M3：完成回退拒绝测试覆盖。
  - M4：完成验证、总项目文档与 devlog 更新。
- Technical Risks:
  - **比较语义风险**：`since_node_id` 依赖字典序，若 consumer 使用不同规范化策略，可能造成推进顺序歧义。
  - **历史数据兼容风险**：旧状态数据缺失字段时需保持兼容默认值，避免误判为回退。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-025-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-025-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
