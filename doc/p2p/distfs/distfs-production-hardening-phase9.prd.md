# Agent World Runtime：DistFS 生产化增强（Phase 9）设计文档

审计轮次: 5
## ROUND-002 主从口径
- 主入口为 `distfs-production-hardening-phase1.prd.md`，本文仅维护阶段增量。

## 1. Executive Summary
- Problem Statement: 为 DistFS 自适应挑战调度补充退避决策级别可观测数据（backoff observability），便于生产排障与参数调优。
- Proposed Solution: 在保持向后兼容的前提下扩展 probe cursor 状态结构，保留旧状态文件可恢复能力。
- Success Criteria:
  - SC-1: 将新增观测字段接入现有测试闭环，确保 runtime 持久化与报告链路可稳定消费。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 生产化增强（Phase 9）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH9-1：文档与任务拆解**
  - AC-2: 输出 Phase 9 设计文档与项目管理文档。
  - AC-3: **DPH9-2：调度状态观测字段扩展（DistFS）**
  - AC-4: 扩展 `StorageChallengeProbeCursorState`，新增退避观测字段：
  - AC-5: `cumulative_backoff_skipped_rounds`
  - AC-6: `cumulative_backoff_applied_ms`
- Non-Goals:
  - 新增 reward 结算公式。
  - 引入新的 challenge 类型或证明系统。
  - 分布式集中调度器（orchestrator）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase9.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase9.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 状态结构扩展（草案）
```rust
StorageChallengeProbeCursorState {
  next_blob_cursor: usize,
  rounds_executed: u64,
  cumulative_total_checks: u64,
  cumulative_passed_checks: u64,
  cumulative_failed_checks: u64,
  cumulative_failure_reasons: BTreeMap<String, u64>,
  consecutive_failure_rounds: u64,
  backoff_until_unix_ms: i64,
  last_probe_unix_ms: Option<i64>,
  cumulative_backoff_skipped_rounds: u64,
  cumulative_backoff_applied_ms: i64,
  last_backoff_duration_ms: i64,
  last_backoff_reason: Option<String>,
  last_backoff_multiplier: u32,
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH9-M1**：文档与任务拆解完成。
  - **DPH9-M2**：调度状态扩展与行为接线完成。
  - **DPH9-M3**：runtime 单测与序列化覆盖完成。
  - **DPH9-M4**：回归与文档收口完成。
- Technical Risks:
  - 状态字段增长可能带来维护成本；通过命名约束与测试覆盖控制。
  - 观测字段若更新时机不一致会误导运维；通过行为单测锁定语义。
  - 回写逻辑若处理不当可能破坏兼容；通过 `serde(default)` 与 legacy 反序列化测试兜底。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-075-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-075-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
