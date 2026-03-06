# Agent World Runtime：DistFS 生产化增强（Phase 3）设计文档

审计轮次: 3

## ROUND-002 主从口径
- 主入口文档：`doc/p2p/distfs/distfs-production-hardening-phase1.prd.md`。
- 本文件为 Phase 3 增量子文档（slave），仅维护本阶段增量内容。

## 1. Executive Summary
- Problem Statement: 将 DistFS 挑战机制从“库内能力”推进到“reward runtime 可消费”的生产闭环。
- Proposed Solution: 让 `storage_valid_checks/storage_total_checks` 使用真实挑战结果，而非固定启发式计数。
- Success Criteria:
  - SC-1: 保持兼容：无可挑战数据时不阻断结算主链路。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 生产化增强（Phase 3）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH3-1：文档与任务拆解**
  - AC-2: 输出 Phase 3 设计文档与项目管理文档。
  - AC-3: **DPH3-2：DistFS 挑战探测接口（Probe）**
  - AC-4: 在 `agent_world_distfs` 增加“按 tick 发起挑战并汇总统计”的统一入口。
  - AC-5: 输出节点维度挑战统计（total/pass/fail + 失败原因分布）。
  - AC-6: **DPH3-3：reward runtime 接线真实挑战计数**
- Non-Goals:
  - 多节点网络挑战调度器。
  - 链上罚没执行与治理参数上链。
  - ZK/PoRep/PoSt 证明系统接入。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase3.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase3.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### DistFS Probe（草案）
```rust
StorageChallengeProbeConfig {
  max_sample_bytes: u32,
  challenges_per_tick: u32,
  challenge_ttl_ms: i64,
  allowed_clock_skew_ms: i64,
}

StorageChallengeProbeReport {
  node_id: String,
  world_id: String,
  observed_at_unix_ms: i64,
  total_checks: u64,
  passed_checks: u64,
  failed_checks: u64,
  failure_reasons: BTreeMap<String, u64>,
  latest_proof_semantics: Option<StorageChallengeProofSemantics>,
}

LocalCasStore::probe_storage_challenges(
  world_id: &str,
  node_id: &str,
  observed_at_unix_ms: i64,
  config: &StorageChallengeProbeConfig,
) -> Result<StorageChallengeProbeReport, WorldError>
```

### Reward Runtime 接线（草案）
- 每轮 `reward_runtime_loop`：
  - 对 `storage_root` 执行 probe；
  - 将 `passed/total` 注入 `NodePointsRuntimeObservation`；
  - 把 probe 报告写入 epoch JSON 报告字段（`distfs_challenge_report`）。

### 兼容语义
- 若 probe 失败或无样本，保留既有 observation 逻辑，不阻断 reward settlement。

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH3-M1**：文档与任务拆解完成。
  - **DPH3-M2**：DistFS Probe 能力与单测完成。
  - **DPH3-M3**：reward runtime 接线与单测完成。
  - **DPH3-M4**：回归与文档收口完成。
- Technical Risks:
  - 挑战频率过高会增加 I/O 压力；需默认保守参数（低频小样本）。
  - 节点无 blob 时统计为 0，可能导致短期奖励偏低；需在配置侧配合挑战调度策略。
  - 回执失败原因分类若过粗，会降低运维定位效率；本期先提供稳定枚举映射，后续细化。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-069-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-069-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
