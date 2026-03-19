# oasis7 Runtime：DistFS 生产化增强（Phase 2）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase2.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase2.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口文档：`doc/p2p/distfs/distfs-production-hardening-phase1.prd.md`。
- 本文件为 Phase 2 增量子文档（slave），仅维护本阶段增量内容。

## 1. Executive Summary
- Problem Statement: 在 `agent_world_distfs` 增加可验证存储挑战（Storage Challenge）闭环能力，为“存储节点收益分配”提供可审计输入。
- Proposed Solution: 保持 DistFS 本地 CAS 语义与现有复制路径兼容，不引入破坏性接口变更。
- Success Criteria:
  - SC-1: 输出可被 reward runtime/链上结算层直接消费的挑战证明语义与统计结果。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 生产化增强（Phase 2）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH2-1：挑战任务模型**
  - AC-2: 新增挑战请求/挑战任务/挑战回执数据结构。
  - AC-3: 定义版本号与 proof kind，便于后续协议演进。
  - AC-4: **DPH2-2：挑战下发与应答**
  - AC-5: `LocalCasStore` 支持基于 `content_hash` 下发挑战：
  - AC-6: 校验 blob 存在与哈希完整性；
- Non-Goals:
  - 分布式 VRF 共识挑战调度网络。
  - 链上惩罚执行与经济参数治理。
  - 零知识证明/PoRep/PoSt 全协议引入。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase2.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase2.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 挑战请求与任务（草案）
```rust
StorageChallengeRequest {
  challenge_id: String,
  world_id: String,
  node_id: String,
  content_hash: String,
  max_sample_bytes: u32,
  issued_at_unix_ms: i64,
  challenge_ttl_ms: i64,
  vrf_seed: String,
}

StorageChallenge {
  version: u64,
  challenge_id: String,
  world_id: String,
  node_id: String,
  content_hash: String,
  sample_offset: u64,
  sample_size_bytes: u32,
  expected_sample_hash: String,
  issued_at_unix_ms: i64,
  expires_at_unix_ms: i64,
  vrf_seed: String,
}
```

### 挑战回执与校验（草案）
```rust
StorageChallengeReceipt {
  version: u64,
  challenge_id: String,
  node_id: String,
  content_hash: String,
  sample_offset: u64,
  sample_size_bytes: u32,
  sample_hash: String,
  responded_at_unix_ms: i64,
  sample_source: StorageChallengeSampleSource,
  failure_reason: Option<StorageChallengeFailureReason>,
  proof_kind: String,
}

verify_storage_challenge_receipt(challenge, receipt, allowed_clock_skew_ms)
```

### 统计输出（草案）
```rust
NodeStorageChallengeStats {
  node_id: String,
  total_checks: u64,
  passed_checks: u64,
  failed_checks: u64,
  failures_by_reason: BTreeMap<StorageChallengeFailureReason, u64>,
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH2-M1**：文档与任务拆解完成。
  - **DPH2-M2**：挑战下发/应答落地。
  - **DPH2-M3**：回执验真与语义投影落地。
  - **DPH2-M4**：统计聚合与测试落地。
  - **DPH2-M5**：回归与文档收口。
- Technical Risks:
  - 挑战采样窗口算法若不稳定会导致跨节点结果不一致；需固定确定性算法与编码。
  - 时间窗校验若过严可能误判时钟偏差；通过 `allowed_clock_skew_ms` 参数控制。
  - 当前仍是样本哈希挑战，不等价于完整 PoRep/PoSt；需在后续阶段扩展更强证明协议。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-068-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-068-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
