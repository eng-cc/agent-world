# oasis7 Runtime：DistFS 生产化增强（Phase 4）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase4.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase4.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口文档：`doc/p2p/distfs/distfs-production-hardening-phase1.prd.md`。
- 本文件为 Phase 4 增量子文档（slave），仅维护本阶段增量内容。

## 1. Executive Summary
- Problem Statement: 在 DistFS 挑战探测链路引入“有状态调度”，避免每轮重复命中同一批 blob，提高挑战覆盖率与公平性。
- Proposed Solution: 为 reward runtime 增加挑战调度状态持久化与恢复能力，保证重启后探测序列连续。
- Success Criteria:
  - SC-1: 保持现有 reward settlement 主链路兼容：挑战路径异常时不阻断结算。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want oasis7 Runtime：DistFS 生产化增强（Phase 4）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH4-1：文档与任务拆解**
  - AC-2: 输出 Phase 4 设计文档与项目管理文档。
  - AC-3: **DPH4-2：DistFS 有状态挑战调度接口**
  - AC-4: 新增调度状态模型（cursor + 累计统计）。
  - AC-5: 新增 `LocalCasStore::probe_storage_challenges_with_cursor(...)`：
  - AC-6: 按 cursor 轮转选择 blob；
- Non-Goals:
  - 跨节点统一挑战调度器（网络级 coordinator）。
  - 链上惩罚执行与治理参数自动调整。
  - ZK/PoRep/PoSt 证明协议升级。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase4.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase4.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 调度状态（草案）
```rust
StorageChallengeProbeCursorState {
  next_blob_cursor: usize,
  rounds_executed: u64,
  cumulative_total_checks: u64,
  cumulative_passed_checks: u64,
  cumulative_failed_checks: u64,
  cumulative_failure_reasons: BTreeMap<String, u64>,
}
```

### 有状态探测（草案）
```rust
LocalCasStore::probe_storage_challenges_with_cursor(
  world_id: &str,
  node_id: &str,
  observed_at_unix_ms: i64,
  config: &StorageChallengeProbeConfig,
  state: &mut StorageChallengeProbeCursorState,
) -> Result<StorageChallengeProbeReport, WorldError>
```

### reward runtime 接线（草案）
- 新增状态文件：`reward-runtime-distfs-probe-state.json`。
- 生命周期：启动加载 -> 每轮更新 -> 原子写回。

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH4-M1**：文档与任务拆解完成。
  - **DPH4-M2**：DistFS 有状态探测能力完成。
  - **DPH4-M3**：reward runtime 状态持久化接线完成。
  - **DPH4-M4**：回归与文档收口完成。
- Technical Risks:
  - 若 blob 集合动态变化较快，cursor 可能短期跳跃；通过 `% blob_count` 约束和累计统计降低影响。
  - 状态文件损坏会导致探测重置；通过容错加载（失败回退默认状态）保证服务可用。
  - 高频持久化增加 I/O；当前每轮一写可接受，后续可加节流。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-070-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-070-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
