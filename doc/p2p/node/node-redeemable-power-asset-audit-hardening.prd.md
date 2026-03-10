# Agent World Runtime：可兑现节点资产与电力兑换闭环（二期审计与签名加固）

- 对应设计文档: `doc/p2p/node/node-redeemable-power-asset-audit-hardening.design.md`
- 对应项目管理文档: `doc/p2p/node/node-redeemable-power-asset-audit-hardening.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口文档：`doc/p2p/node/node-redeemable-power-asset.prd.md`。
- 本文件仅维护“二期审计与签名加固”增量专题约束。
- 通用规格以主文档为准。

## 1. Executive Summary
- Problem Statement: 在已完成 `PowerCredit -> Agent 电力` 兑换闭环的基础上，补齐可审计性与签名语义，降低账本漂移和伪造记录风险。
- Proposed Solution: 将当前结算记录中的占位签名（`bound:<public_key>`）升级为可重算、可校验的结构化签名语义。
- Success Criteria:
  - SC-1: 提供运行时资产守恒巡检能力，支持在 `test_tier_required` 下快速发现资产状态异常。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：可兑现节点资产与电力兑换闭环（二期审计与签名加固） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **AHA-1：结算签名语义升级（V1）**
  - AC-2: 定义结算铸造记录签名串格式：`mintsig:v1:<sha256_hex>`。
  - AC-3: 签名摘要输入至少包含：`epoch_index`、`node_id`、`source_awarded_points`、`minted_power_credits`、`settlement_hash`、`signer_node_id`、`signer_public_key`。
  - AC-4: 新增签名计算与校验函数，并在 `apply_node_points_settlement_mint` 中产出该格式签名。
  - AC-5: **AHA-2：账本不变量审计报告（Runtime API）**
  - AC-6: 新增 `RewardAssetInvariantReport`，输出聚合统计与异常列表。
- Non-Goals:
  - 完整公私钥签名体系（真实私钥签名、链上验签 gas/成本模型）。
  - 复杂治理流程（仲裁委员会、多签提案、自动罚没执行策略）。
  - 经济参数自适应调控（动态汇率、拍卖、AMM）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-redeemable-power-asset-audit-hardening.prd.md`
  - `doc/p2p/node/node-redeemable-power-asset-audit-hardening.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) 结算签名语义（草案）
```rust
// 序列化后拼接字符串（v1）
format!(
  "{}|{}|{}|{}|{}|{}|{}",
  epoch_index,
  node_id,
  source_awarded_points,
  minted_power_credits,
  settlement_hash,
  signer_node_id,
  signer_public_key,
)

// signature = "mintsig:v1:" + sha256_hex(payload_bytes)
```

### 2) 审计报告（草案）
```rust
RewardAssetInvariantViolation {
  code: String,
  message: String,
}

RewardAssetInvariantReport {
  total_nodes: usize,
  total_minted_credits: u64,
  total_burned_credits: u64,
  total_power_credit_balance: u64,
  mint_record_count: usize,
  violations: Vec<RewardAssetInvariantViolation>,
}
```

### 3) Runtime API（草案）
```rust
impl World {
  pub fn reward_asset_invariant_report(&self) -> RewardAssetInvariantReport;
  pub fn verify_reward_mint_record_signature(&self, record: &NodeRewardMintRecord) -> Result<(), String>;
}
```

#### 测试策略
- `test_tier_required`：
  - 结算签名生成/验签通过；
  - 伪造签名、签名者未绑定场景被识别；
  - 守恒报告在正常与篡改样本下输出预期。
- `test_tier_full`：
  - 多节点多 epoch 结算后不变量巡检；
  - 快照恢复后审计结果一致；
  - reward runtime 周期报告持续输出审计摘要。

## 5. Risks & Roadmap
- Phased Rollout:
  - **AHA-M0**：设计文档与项目管理文档完成。
  - **AHA-M1**：结算签名语义升级与校验函数落地。
  - **AHA-M2**：账本不变量审计报告 API 与单测闭环。
  - **AHA-M3**：`world_viewer_live` 报表接线与回归。
  - **AHA-M4**：文档/devlog/发布说明收口。
- Technical Risks:
  - V1 签名语义仍为“可重算摘要”，并非真实私钥签名，安全强度受限于运行环境信任边界。
  - 审计报告为“检测”而非“自动纠正”，需要配合治理动作处理异常。
  - 若后续升级真实签名算法，需保证与 `mintsig:v1` 的兼容迁移策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-096-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-096-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
