# Agent World Runtime：可兑现节点资产与电力兑换闭环（三期真实签名与治理闭环）

## 目标
- 在二期 `mintsig:v1` 可重算摘要的基础上，落地真实私钥签名 `mintsig:v2`（ed25519），使结算铸造记录具备密码学不可抵赖性。
- 为兑换动作补齐签名治理闭环：支持签名版兑换动作、验签与治理策略开关，形成“结算签名 + 兑换签名”统一治理路径。
- 保持现有账本与快照兼容，支持在治理策略控制下平滑从 `mintsig:v1` 迁移到 `mintsig:v2`。

## 范围

### In Scope
- **SGC-1：结算签名升级到 `mintsig:v2`**
  - 新增 ed25519 签名载荷规范与签名串格式：`mintsig:v2:<signature_hex>`。
  - 新增 `apply_node_points_settlement_mint_v2(...)`，由真实私钥对结算记录签名。
  - `verify_reward_mint_record_signature` 支持 v1/v2 双轨验签。

- **SGC-2：签名治理策略**
  - 新增 `RewardSignatureGovernancePolicy`：
    - `require_mintsig_v2`：是否强制结算必须使用 v2。
    - `allow_mintsig_v1_fallback`：是否允许历史 v1 验签通过。
    - `require_redeem_signature`：是否强制兑换必须携带签名。
  - 在 runtime 结算/兑换入口接入策略检查。

- **SGC-3：兑换签名动作闭环**
  - 新增签名版动作 `Action::RedeemPowerSigned`（含 `signer_node_id` 与 `signature`）。
  - 新增 `redeemsig:v1` 签名与验签函数（ed25519）。
  - 当策略要求兑换签名时，未签名兑换动作被拒绝并输出明确拒绝原因。

- **SGC-4：reward runtime 主链路接线**
  - `world_viewer_live` reward runtime 使用配置私钥产出 `mintsig:v2`。
  - auto redeem 改为提交签名版兑换动作，形成最小运行闭环。

### Out of Scope
- 多签治理（M-of-N）与委员会仲裁流程。
- 链上 gas/费用模型与签名成本优化。
- 跨节点密钥托管平台、HSM、远程签名服务。

## 接口 / 数据

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

## 里程碑
- **SGC-M0**：设计文档与项目管理文档完成。
- **SGC-M1**：`mintsig:v2` 签名/验签与治理策略落地。
- **SGC-M2**：签名版兑换动作与策略门禁落地。
- **SGC-M3**：`world_viewer_live` reward runtime 接入真实私钥签名闭环。
- **SGC-M4**：`test_tier_required` 回归、文档状态回写与 devlog 收口。

## 测试策略
- `test_tier_required`：
  - `mintsig:v2` 生成/验签通过；篡改记录验签失败。
  - 策略开启后拒绝无签名兑换与 v1 结算（当强制 v2 时）。
  - 签名版兑换动作成功路径与拒绝路径覆盖。
- `test_tier_full`：
  - 多 epoch 结算/兑换混合场景下审计报告稳定。
  - 快照恢复后签名治理策略与验签行为一致。

## 风险
- 私钥输入为 hex 字符串，若运维流程泄露会带来密钥风险；三期只做最小闭环，不引入 HSM。
- 治理策略配置错误（例如强制签名但未正确绑定公钥）会导致兑换/结算被全量拒绝。
- v1/v2 共存期需要严格口径，避免策略与历史数据兼容预期不一致。
