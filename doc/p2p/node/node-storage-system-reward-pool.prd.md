# Agent World Runtime：节点存储系统奖励池

## 1. Executive Summary
- Problem Statement: 在 Node Points 结算中新增“存储系统奖励池”，使存储奖励和在线奖励一样由协议固定池发放。
- Proposed Solution: 存储奖励以挑战通过率为核心资格，避免仅靠自报容量领取奖励。
- Success Criteria:
  - SC-1: 在保持现有结算接口可兼容的前提下，提供生产可用的可观测字段（主池/存储池拆分）。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点存储系统奖励池 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `NodePointsConfig` 新增存储池与反作弊参数：
  - AC-2: `storage_pool_points`
  - AC-3: `min_storage_challenge_pass_ratio`
  - AC-4: `min_storage_challenge_checks`
  - AC-5: `max_rewardable_storage_to_staked_ratio`（0 表示关闭按质押封顶）
  - AC-6: 在 `NodeContributionSample` 新增存储挑战/质押字段：
- Non-Goals:
  - 链上质押资产真实扣罚（本次仅接入样本字段和封顶约束）。
  - 跨节点挑战协议、VRF 随机挑战网络实现。
  - 需求侧订单支付市场（本次聚焦系统奖励池）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-storage-system-reward-pool.prd.md`
  - `doc/p2p/node/node-storage-system-reward-pool.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 配置新增字段
```rust
NodePointsConfig {
  storage_pool_points: u64,
  min_storage_challenge_pass_ratio: f64,
  min_storage_challenge_checks: u64,
  // 可奖励存储上限 = staked_storage_bytes * ratio；0 表示不封顶
  max_rewardable_storage_to_staked_ratio: f64,
}
```

### 样本新增字段
```rust
NodeContributionSample {
  storage_valid_checks: u64,
  storage_total_checks: u64,
  staked_storage_bytes: u64,
}
```

### 结算新增字段
```rust
NodeSettlement {
  storage_reward_score: f64,
  rewardable_storage_bytes: u64,
  main_awarded_points: u64,
  storage_awarded_points: u64,
  awarded_points: u64, // 总奖励
}

EpochSettlementReport {
  pool_points: u64,            // 主池
  storage_pool_points: u64,    // 存储池
  distributed_points: u64,     // 主池已分配
  storage_distributed_points: u64,
  total_distributed_points: u64,
}
```

### 结算规则（存储池）
- 挑战通过率：`storage_pass_ratio = valid / total`（`total > 0`）。
- 资格门槛：
  - `total >= min_storage_challenge_checks`；
  - `storage_pass_ratio > min_storage_challenge_pass_ratio`。
- 通过率归一化：
  - `norm = max(0, (pass - min_ratio) / (1 - min_ratio))`。
- 可奖励存储：
  - 默认 `rewardable = effective_storage_bytes`；
  - 若 `max_rewardable_storage_to_staked_ratio > 0` 且 `staked_storage_bytes > 0`，
    `rewardable = min(effective_storage_bytes, staked_storage_bytes * ratio)`。
- 存储池评分：
  - `storage_reward_score = sqrt(rewardable_gib) * norm * availability_ratio`。

## 5. Risks & Roadmap
- Phased Rollout:
  - SBR-1：设计文档与项目管理文档。
  - SBR-2：`node_points` 双池结算与测试。
  - SBR-3：`node_points_runtime` 存储挑战采样接线与测试。
  - SBR-4：`test_tier_required` 回归、文档与 devlog 收口。
- Technical Risks:
  - 若挑战次数不足，存储池分配可能频繁空池；需后续配合挑战调度频率。
  - 质押封顶参数配置不当会抑制真实大节点贡献，需按网络规模调优。
  - 主池仍包含 `weight_storage` 时可能与存储池形成叠加激励，部署时应结合参数策略（例如将主池 `weight_storage` 降低或置零）。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-102-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-102-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
