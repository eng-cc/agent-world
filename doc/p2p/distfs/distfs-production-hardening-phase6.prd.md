# oasis7 Runtime：DistFS 生产化增强（Phase 6）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase6.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase6.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口为 `distfs-production-hardening-phase1.prd.md`，本文仅维护阶段增量。

## 1. Executive Summary
- Problem Statement: 为 DistFS 有状态挑战调度增加自适应失败退避（backoff）能力，减少持续失败时的 I/O 抖动与无效探测。
- Proposed Solution: 引入每轮挑战预算上限，避免在高密度 blob 场景下单轮探测放大。
- Success Criteria:
  - SC-1: 保持历史状态文件兼容，确保线上升级不会因状态字段演进而启动失败。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want oasis7 Runtime：DistFS 生产化增强（Phase 6）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH6-1：文档与任务拆解**
  - AC-2: 输出 Phase 6 设计文档与项目管理文档。
  - AC-3: **DPH6-2：自适应策略与预算上限**
  - AC-4: 在 `challenge_scheduler` 新增自适应调度策略结构。
  - AC-5: 新增 `probe_storage_challenges_with_policy(...)`：
  - AC-6: 按策略限制单轮挑战数量；
- Non-Goals:
  - 多节点统一协调器与跨节点预算仲裁。
  - 链上动态参数治理。
  - ZK/PoRep/PoSt 升级。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase6.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase6.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 自适应策略（草案）
```rust
StorageChallengeAdaptivePolicy {
  max_checks_per_round: u32,
  failure_backoff_base_ms: i64,
  failure_backoff_max_ms: i64,
}
```

### 状态扩展（草案）
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
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH6-M1**：文档与任务拆解完成。
  - **DPH6-M2**：自适应策略与预算上限完成。
  - **DPH6-M3**：状态兼容与测试完成。
  - **DPH6-M4**：回归与文档收口完成。
- Technical Risks:
  - 退避配置不合理可能导致探测过稀；通过默认策略保持“退避关闭”兼容。
  - 状态字段扩展若无默认值会破坏旧状态恢复；通过 `serde(default)` 强约束。
  - 预算限制过低会影响挑战覆盖率；后续可通过 Phase 5 CLI 参数联动调优。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-072-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-072-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
