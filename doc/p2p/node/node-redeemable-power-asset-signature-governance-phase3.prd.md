# Agent World Runtime：可兑现节点资产与电力兑换闭环（三期真实签名与治理闭环）

审计轮次: 2

## ROUND-002 主从口径
- 主入口文档：`doc/p2p/node/node-redeemable-power-asset.prd.md`。
- 本文件仅维护“三期真实签名与治理闭环”增量专题约束。
- 通用规格以主文档为准。

## 1. Executive Summary
- Problem Statement: 在二期 `mintsig:v1` 可重算摘要的基础上，落地真实私钥签名 `mintsig:v2`（ed25519），使结算铸造记录具备密码学不可抵赖性。
- Proposed Solution: 为兑换动作补齐签名治理闭环：支持签名版兑换动作、验签与治理策略开关，形成“结算签名 + 兑换签名”统一治理路径。
- Success Criteria:
  - SC-1: 保持现有账本与快照兼容，支持在治理策略控制下平滑从 `mintsig:v1` 迁移到 `mintsig:v2`。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：可兑现节点资产与电力兑换闭环（三期真实签名与治理闭环） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **SGC-1：结算签名升级到 `mintsig:v2`**
  - AC-2: 新增 ed25519 签名载荷规范与签名串格式：`mintsig:v2:<signature_hex>`。
  - AC-3: 新增 `apply_node_points_settlement_mint_v2(...)`，由真实私钥对结算记录签名。
  - AC-4: `verify_reward_mint_record_signature` 支持 v1/v2 双轨验签。
  - AC-5: **SGC-2：签名治理策略**
  - AC-6: 新增 `RewardSignatureGovernancePolicy`：
- Non-Goals:
  - 多签治理（M-of-N）与委员会仲裁流程。
  - 链上 gas/费用模型与签名成本优化。
  - 跨节点密钥托管平台、HSM、远程签名服务。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-redeemable-power-asset-signature-governance-phase3.prd.md`
  - `doc/p2p/node/node-redeemable-power-asset-signature-governance-phase3.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) 结算签名（V2）
```rust
// 结算签名 payload（示意）
"mintsig:v2|epoch_index|node_id|source_awarded_points|minted_power_credits|settlement_hash|signer_node_id|signer_public_key"

// signature = ed25519_sign(private_key, payload_bytes)
// encoded = "mintsig:v2:" + hex(signature_bytes)
```

### 2) 兑换签名（V1）
```rust
// 兑换签名 payload（示意）
"redeemsig:v1|node_id|target_agent_id|redeem_credits|nonce|signer_node_id|signer_public_key"

// signature = ed25519_sign(private_key, payload_bytes)
// encoded = "redeemsig:v1:" + hex(signature_bytes)
```

### 3) 治理策略（草案）
```rust
RewardSignatureGovernancePolicy {
  require_mintsig_v2: bool,
  allow_mintsig_v1_fallback: bool,
  require_redeem_signature: bool,
}
```

### 4) Runtime API（草案）
```rust
impl World {
  pub fn reward_signature_governance_policy(&self) -> &RewardSignatureGovernancePolicy;
  pub fn set_reward_signature_governance_policy(&mut self, policy: RewardSignatureGovernancePolicy);

  pub fn apply_node_points_settlement_mint_v2(
    &mut self,
    report: &EpochSettlementReport,
    signer_node_id: &str,
    signer_private_key_hex: &str,
  ) -> Result<Vec<NodeRewardMintRecord>, WorldError>;
}
```

#### 测试策略
- `test_tier_required`：
  - `mintsig:v2` 生成/验签通过；篡改记录验签失败。
  - 策略开启后拒绝无签名兑换与 v1 结算（当强制 v2 时）。
  - 签名版兑换动作成功路径与拒绝路径覆盖。
- `test_tier_full`：
  - 多 epoch 结算/兑换混合场景下审计报告稳定。
  - 快照恢复后签名治理策略与验签行为一致。

## 5. Risks & Roadmap
- Phased Rollout:
  - **SGC-M0**：设计文档与项目管理文档完成。
  - **SGC-M1**：`mintsig:v2` 签名/验签与治理策略落地。
  - **SGC-M2**：签名版兑换动作与策略门禁落地。
  - **SGC-M3**：`world_viewer_live` reward runtime 接入真实私钥签名闭环。
  - **SGC-M4**：`test_tier_required` 回归、文档状态回写与 devlog 收口。
- Technical Risks:
  - 私钥输入为 hex 字符串，若运维流程泄露会带来密钥风险；三期只做最小闭环，不引入 HSM。
  - 治理策略配置错误（例如强制签名但未正确绑定公钥）会导致兑换/结算被全量拒绝。
  - v1/v2 共存期需要严格口径，避免策略与历史数据兼容预期不一致。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-097-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-097-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
