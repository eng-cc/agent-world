# Agent World Runtime：节点奖励运行时生产化加固（Phase 1）设计文档

## 1. Executive Summary
- Problem Statement: 将现有奖励链路从“演示可跑”提升为“可持续运行、可恢复、可审计”的生产化基础能力。
- Proposed Solution: 补齐奖励运行时状态持久化，避免进程重启导致 epoch/累计积分/采样窗口丢失。
- Success Criteria:
  - SC-1: 收口兑换签名授权语义，支持策略化强制 `signer_node_id == node_id`，防止托管式滥签默认放开。
  - SC-2: 移除 reward runtime 中的占位身份绑定行为，改为显式绑定来源，避免伪身份进入账本。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点奖励运行时生产化加固（Phase 1）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **PRH1-1：奖励采样状态持久化**
  - AC-2: 为 `NodePointsLedger` 增加可序列化快照结构。
  - AC-3: 为 `NodePointsRuntimeCollector` 增加快照导出/恢复能力。
  - AC-4: 为 `world_viewer_live` reward runtime 增加状态文件落盘与启动恢复。
  - AC-5: **PRH1-2：兑换签名授权收口**
  - AC-6: 在 `RewardSignatureGovernancePolicy` 新增 `require_redeem_signer_match_node_id`。
- Non-Goals:
  - 完整多节点链上奖励结算调度器（独立可执行进程与网络共识提案）。
  - 真实 PoRep/PoSt/VRF 挑战协议及跨观察点拜占庭验证。
  - 多签/HSM/KMS 托管体系与跨组织授权治理。
  - 需求侧订单撮合市场和动态价格机制。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-reward-runtime-production-hardening-phase1.prd.md`
  - `doc/p2p/node/node-reward-runtime-production-hardening-phase1.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) `RewardSignatureGovernancePolicy` 新增字段
```rust
RewardSignatureGovernancePolicy {
  require_mintsig_v2: bool,
  allow_mintsig_v1_fallback: bool,
  require_redeem_signature: bool,
  // true 时强制 signer_node_id == node_id
  require_redeem_signer_match_node_id: bool,
}
```

### 2) Node Points 账本快照（新增）
```rust
NodePointsLedgerSnapshot {
  config: NodePointsConfig,
  epoch_index: u64,
  cumulative_points: BTreeMap<String, u64>,
}
```

### 3) Collector 快照（新增）
```rust
NodePointsRuntimeCollectorSnapshot {
  ledger: NodePointsLedgerSnapshot,
  heuristics: NodePointsRuntimeHeuristics,
  epoch_started_at_unix_ms: Option<i64>,
  cursors: BTreeMap<String, NodeCursor>,
  current_epoch: BTreeMap<String, NodeEpochAccumulator>,
}
```

### 4) Reward Runtime 状态文件（新增）
- 路径：`<reward_runtime_report_dir>/reward-runtime-state.json`
- 内容：`NodePointsRuntimeCollectorSnapshot`。
- 语义：启动优先加载，运行中按固定频率（每次采样）原子写入。

## 5. Risks & Roadmap
- Phased Rollout:
  - **PRH1-M1**：设计/项目文档落地。
  - **PRH1-M2**：collector/ledger 快照序列化与恢复实现。
  - **PRH1-M3**：兑换签名授权策略字段 + 校验门禁实现。
  - **PRH1-M4**：reward runtime 身份绑定行为收口。
  - **PRH1-M5**：测试回归、文档状态更新、devlog 收口。
- Technical Risks:
  - 状态文件损坏可能导致 runtime 恢复失败，需要安全回退为“空状态启动”。
  - 强制 `signer==node` 策略开启后，历史托管签名流程将被拒绝，需要灰度切换。
  - 高频状态落盘会增加 I/O 压力，Phase 1 先保证一致性，后续再做节流优化。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-100-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-100-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
