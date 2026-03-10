# Agent World Runtime：节点贡献积分激励

- 对应设计文档: `doc/p2p/node/node-contribution-points.design.md`
- 对应项目管理文档: `doc/p2p/node/node-contribution-points.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口文档：`doc/p2p/node/node-contribution-points.prd.md`。
- 从文档：`node-contribution-points-runtime-closure.prd.md`、`node-contribution-points-multi-node-closure-test.prd.md` 仅维护增量约束与专题闭环，主规格以本文件为准。

## 1. Executive Summary
- Problem Statement: 在 Agent World 的区块链 + P2P FS 闭环内，引入可审计的节点积分激励（Node Points）。
- Proposed Solution: 明确“基础义务”和“额外贡献”的边界：
- Success Criteria:
  - SC-1: 为自身 Agent 提供模拟计算属于基础义务，不直接奖励；
  - SC-2: 为离线节点代跑模拟、执行世界维护任务属于额外计算，应获得奖励。
  - SC-3: 为长期在线且提供更多有效存储的节点提供额外收益。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点贡献积分激励 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增节点积分结算引擎（epoch 级）。
  - AC-2: 贡献维度：
  - AC-3: `delegated_sim_compute_units`（代跑离线节点）；
  - AC-4: `world_maintenance_compute_units`（世界维护任务）；
  - AC-5: `effective_storage_bytes`（有效存储）；
  - AC-6: `uptime_seconds`（在线时长）；
- Non-Goals:
  - 链上可交易代币、真实经济清算。
  - 完整质押/罚没资产系统（仅保留积分惩罚入口）。
  - 复杂证明协议（PoRep/PoSt/ZK）的真实网络接线。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-contribution-points.prd.md`
  - `doc/p2p/node/node-contribution-points.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 核心配置（草案）
```rust
NodePointsConfig {
  epoch_duration_seconds: u64,
  epoch_pool_points: u64,
  min_self_sim_compute_units: u64,
  delegated_compute_multiplier: f64,
  maintenance_compute_multiplier: f64,
  weight_compute: f64,
  weight_storage: f64,
  weight_uptime: f64,
  weight_reliability: f64,
  obligation_penalty_points: f64,
}
```

### 节点贡献输入（草案）
```rust
NodeContributionSample {
  node_id: String,
  self_sim_compute_units: u64,
  delegated_sim_compute_units: u64,
  world_maintenance_compute_units: u64,
  effective_storage_bytes: u64,
  uptime_seconds: u64,
  verify_pass_ratio: f64,
  availability_ratio: f64,
  explicit_penalty_points: f64,
}
```

### 结算输出（草案）
```rust
NodeSettlement {
  node_id: String,
  obligation_met: bool,
  compute_score: f64,
  storage_score: f64,
  uptime_score: f64,
  reliability_score: f64,
  penalty_score: f64,
  total_score: f64,
  awarded_points: u64,
  cumulative_points: u64,
}

EpochSettlementReport {
  epoch_index: u64,
  pool_points: u64,
  distributed_points: u64,
  settlements: Vec<NodeSettlement>,
}
```

### 计分公式（MVP）
- 额外计算分：
  - `compute_units = delegated * delegated_multiplier + maintenance * maintenance_multiplier`
  - `compute_score = compute_units * verify_pass_ratio`
- 存储分：
  - `storage_gib = effective_storage_bytes / 1024^3`
  - `storage_score = sqrt(storage_gib) * availability_ratio`
- 在线分：
  - `uptime_score = min(1.0, uptime_seconds / epoch_duration_seconds)`
- 可靠性分：
  - `reliability_score = (verify_pass_ratio + availability_ratio) / 2`
- 总分：
  - `total = w_c*compute + w_s*storage + w_u*uptime + w_r*reliability - penalty`
  - `total < 0` 则按 `0` 处理。
- 基础义务惩罚：
  - 当 `self_sim_compute_units < min_self_sim_compute_units` 时，额外加罚 `obligation_penalty_points`。

## 5. Risks & Roadmap
- Phased Rollout:
  - NCP-1：设计文档 + 项目管理文档。
  - NCP-2：节点积分引擎核心实现（计算/存储/在线/惩罚 + 台账）。
  - NCP-3：测试与导出接线（test_tier_required 口径）。
  - NCP-4：文档状态回写与 devlog 收口。
- Technical Risks:
  - 参数不当可能导致单一资源（大存储或大算力）垄断积分，需要通过 `sqrt(storage)` 与权重平衡缓解。
  - 若没有真实证明接线，`verify_pass_ratio/availability_ratio` 的真实性依赖上层采样器，后续需替换为链路证明数据。
  - 积分池固定时，低活跃 epoch 可能出现“有效贡献过少”，需在后续迭代加入最小活跃阈值与回收池机制。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-091-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-091-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
