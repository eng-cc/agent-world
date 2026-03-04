# Agent World Runtime：节点执行桥接与奖励共识触发闭环

## 1. Executive Summary
- Problem Statement: 把 `agent_world_node` 的共识提交高度与 `agent_world` 运行时执行状态打通，补齐“提交高度 -> 执行状态 -> 存储落盘”的桥接链路。
- Proposed Solution: 将奖励结算从本地线程直触发升级为“基于网络传播的结算包触发”，使多节点可消费同一结算动作。
- Success Criteria:
  - SC-1: 为奖励采样引入“多观察者 + 签名轨迹（trace）”输入，降低单观察点自报风险，提升可验证性与审计能力。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点执行桥接与奖励共识触发闭环 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **ERCB-1：节点提交高度到执行状态桥接**
  - AC-2: 在 `world_viewer_live` reward runtime 内引入执行桥接器：
  - AC-3: 监听 `NodeRuntime` 的 `committed_height` 变化；
  - AC-4: 对每个新提交高度驱动一次 `RuntimeWorld` 执行步进；
  - AC-5: 产出并落盘执行记录（execution block record），包含 `height/state_root/block_hash/journal_len`；
  - AC-6: 将快照与日志写入 DistFS CAS（本地存储根）。
- Non-Goals:
  - 跨组织 BFT 奖励提案投票与链上治理。
  - 跨世界分片的奖励汇总与原子清算。
  - 完整 PoRep/PoSt/VRF 协议实现（本期仅提供签名轨迹 + 多观察者输入基础）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-execution-reward-consensus-bridge.prd.md`
  - `doc/p2p/node/node-execution-reward-consensus-bridge.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) 执行桥接状态（新增）
```rust
ExecutionBridgeState {
  last_applied_committed_height: u64,
  last_execution_block_hash: Option<String>,
  last_execution_state_root: Option<String>,
  last_node_block_hash: Option<String>,
}
```

### 2) 执行桥接记录（新增）
```rust
ExecutionBridgeRecord {
  world_id: String,
  height: u64,
  node_block_hash: Option<String>,
  execution_block_hash: String,
  execution_state_root: String,
  journal_len: usize,
  snapshot_ref: String,
  journal_ref: String,
  timestamp_ms: i64,
}
```

### 3) 观察轨迹消息（新增）
```rust
RewardObservationTrace {
  version: u8,
  world_id: String,
  observer_node_id: String,
  observer_public_key_hex: String,
  payload: RewardObservationPayload,
  payload_hash: String,
  signature: String, // rewardobs:v1:<sig_hex>
}
```

### 4) 结算包消息（新增）
```rust
RewardSettlementEnvelope {
  version: u8,
  world_id: String,
  epoch_index: u64,
  signer_node_id: String,
  report: EpochSettlementReport,
  mint_records: Vec<NodeRewardMintRecord>,
  emitted_at_unix_ms: i64,
}
```

### 5) 主题命名（新增）
- `aw.<world_id>.reward.observation`
- `aw.<world_id>.reward.settlement`

### 6) 触发规则
- 执行桥接：`committed_height` 增长时按增量高度逐个桥接。
- 结算发布：collector 到达 epoch 结算点且本节点 `role=sequencer` 时发布。
- 结算应用：收到网络结算包后提交 `ApplyNodePointsSettlementSigned` action 并 `step()`。

## 5. Risks & Roadmap
- Phased Rollout:
  - **ERCB-M0**：设计文档 + 项目管理文档。
  - **ERCB-M1**：执行桥接器实现（状态恢复、记录落盘、CAS 引用写入）。
  - **ERCB-M2**：奖励结算网络触发与结算包应用实现。
  - **ERCB-M3**：多观察者签名轨迹实现与 collector 接线。
  - **ERCB-M4**：测试回归（`test_tier_required`）+ 文档/devlog 收口。
- Technical Risks:
  - 执行桥接是 runtime 外围桥接而非共识内核执行，仍存在“共识 block_hash 与执行 block_hash 不同源”的阶段性差异。
  - 在网络抖动场景下，观察轨迹和结算包可能乱序/重复，需要幂等去重兜底。
  - 若 `--reward-runtime-min-observer-traces` 配置过高，低活跃网络可能延迟结算。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-093-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-093-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
