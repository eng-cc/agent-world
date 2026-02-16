# Agent World Runtime：可兑现节点资产与电力兑换闭环（二期审计与签名加固）

## 目标
- 在已完成 `PowerCredit -> Agent 电力` 兑换闭环的基础上，补齐可审计性与签名语义，降低账本漂移和伪造记录风险。
- 将当前结算记录中的占位签名（`bound:<public_key>`）升级为可重算、可校验的结构化签名语义。
- 提供运行时资产守恒巡检能力，支持在 `test_tier_required` 下快速发现资产状态异常。

## 范围

### In Scope
- **AHA-1：结算签名语义升级（V1）**
  - 定义结算铸造记录签名串格式：`mintsig:v1:<sha256_hex>`。
  - 签名摘要输入至少包含：`epoch_index`、`node_id`、`source_awarded_points`、`minted_power_credits`、`settlement_hash`、`signer_node_id`、`signer_public_key`。
  - 新增签名计算与校验函数，并在 `apply_node_points_settlement_mint` 中产出该格式签名。

- **AHA-2：账本不变量审计报告（Runtime API）**
  - 新增 `RewardAssetInvariantReport`，输出聚合统计与异常列表。
  - 核心不变量至少包含：
    - 单节点 `power_credit_balance == total_minted_credits - total_burned_credits`；
    - `total_burned_credits <= total_minted_credits`；
    - 全局余额守恒（节点余额求和与 mint/burn 汇总一致）；
    - 结算记录签名可校验且签名者绑定存在。

- **AHA-3：最小运行链路接入**
  - 在 `world_viewer_live` reward runtime 报表中增加不变量审计摘要。
  - 保持默认兼容，不改变现有兑换业务路径，仅增强观测和告警。

### Out of Scope
- 完整公私钥签名体系（真实私钥签名、链上验签 gas/成本模型）。
- 复杂治理流程（仲裁委员会、多签提案、自动罚没执行策略）。
- 经济参数自适应调控（动态汇率、拍卖、AMM）。

## 接口 / 数据

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

## 里程碑
- **AHA-M0**：设计文档与项目管理文档完成。
- **AHA-M1**：结算签名语义升级与校验函数落地。
- **AHA-M2**：账本不变量审计报告 API 与单测闭环。
- **AHA-M3**：`world_viewer_live` 报表接线与回归。
- **AHA-M4**：文档/devlog/发布说明收口。

## 测试策略
- `test_tier_required`：
  - 结算签名生成/验签通过；
  - 伪造签名、签名者未绑定场景被识别；
  - 守恒报告在正常与篡改样本下输出预期。
- `test_tier_full`：
  - 多节点多 epoch 结算后不变量巡检；
  - 快照恢复后审计结果一致；
  - reward runtime 周期报告持续输出审计摘要。

## 风险
- V1 签名语义仍为“可重算摘要”，并非真实私钥签名，安全强度受限于运行环境信任边界。
- 审计报告为“检测”而非“自动纠正”，需要配合治理动作处理异常。
- 若后续升级真实签名算法，需保证与 `mintsig:v1` 的兼容迁移策略。
