# Agent World Runtime：节点基础在线时长奖励

## 1. Executive Summary
- Problem Statement: 在现有节点贡献积分（Node Points）中加入“基础在线时长奖励”可运行实现。
- Proposed Solution: 在线奖励基于“挑战通过率”而不是节点自报时长，降低假在线收益。
- Success Criteria:
  - SC-1: 保持与已有计算/存储/可靠性积分模型兼容，不破坏现有结算台账结构。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点基础在线时长奖励 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `NodePointsConfig` 增加在线奖励门槛参数：`min_uptime_challenge_pass_ratio`。
  - AC-2: 在 `NodeContributionSample` 增加在线挑战统计：
  - AC-3: `uptime_valid_checks`
  - AC-4: `uptime_total_checks`
  - AC-5: 在结算逻辑中引入在线得分归一化：
  - AC-6: `raw_uptime_ratio = valid / total`（当 `total > 0`）
- Non-Goals:
  - 链上随机挑战（VRF）和跨节点挑战网络协议。
  - 罚没资金清算与仲裁系统。
  - 多观察点拜占庭容错（本次仅做基础版采样结构）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-uptime-base-reward.prd.md`
  - `doc/p2p/node/node-uptime-base-reward.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 配置新增字段
```rust
NodePointsConfig {
  // 0.0..=1.0，低于该挑战通过率时在线项不计分
  min_uptime_challenge_pass_ratio: f64,
}
```

### 贡献样本新增字段
```rust
NodeContributionSample {
  // 当 epoch 内有挑战记录时，在线得分以挑战统计为准
  uptime_valid_checks: u64,
  uptime_total_checks: u64,
}
```

### Runtime 观察新增字段
```rust
NodePointsRuntimeObservation {
  uptime_checks_passed: u64,
  uptime_checks_total: u64,
}
```

### 结算规则（在线项）
- 若 `uptime_total_checks > 0`，在线率使用挑战通过率；
- 否则使用 `uptime_seconds / epoch_duration_seconds` 作为回退口径；
- 在线奖励门槛通过 `min_uptime_challenge_pass_ratio` 控制，低于门槛不给在线分；
- 达标后按线性归一化给分，最大不超过 1。

## 5. Risks & Roadmap
- Phased Rollout:
  - UBR-1：完成设计文档与项目管理文档。
  - UBR-2：完成 `node_points` 在线挑战奖励实现与测试。
  - UBR-3：完成 `node_points_runtime` 挑战采样接线与测试。
  - UBR-4：完成回归测试、文档状态回写和 devlog 收口。
- Technical Risks:
  - 若挑战频率过低，在线率统计波动会偏大；需后续引入最小挑战数门槛。
  - 目前默认采样仍可被本地进程状态影响，后续应接入多观察点挑战源。
  - 配置 `min_uptime_challenge_pass_ratio` 过高可能导致新节点难以拿到在线分，需结合运营参数调优。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-103-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-103-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
