# Agent World 主链 Token 分配机制二期：地址绑定 + 治理绑定 + 分发闭环（2026-02-26）

## 目标
- 将 NodePoints -> 主链 Token 桥接的收款方从 `account_id = node_id` 升级为真实地址绑定语义。
- 将 `Action::UpdateMainTokenPolicy` 与治理提案生命周期绑定，避免“脱离治理状态”的参数更新。
- 为 `staking_reward_pool`、`ecosystem_pool`、`security_reserve` 提供可审计的分发闭环，避免只入 treasury 无后续主路径动作。

## 范围
### In Scope
- Runtime 状态新增：节点主链地址绑定、主链 treasury 分发审计记录。
- Runtime Action/DomainEvent 新增：主链 treasury 分发动作与事件。
- Runtime 规则增强：
  - 主链策略更新必须绑定到已批准/已应用的治理提案；
  - NodePoints 桥接出账改为使用“节点绑定主链地址”；
  - treasury 分发动作支持治理绑定和幂等审计。
- 查询接口增强：新增主链地址绑定、treasury 分发记录查询。
- `test_tier_required` / `test_tier_full` 回归覆盖更新。

### Out of Scope
- 跨链地址体系、外部钱包协议（只做 runtime 内部地址字符串绑定）。
- 治理模块（manifest governance / gameplay governance）结构重构。
- 新增经济学参数（通胀公式和 split 比例不变）。

## 接口 / 数据
### 1) 状态新增
- `WorldState.node_main_token_account_bindings: BTreeMap<String, String>`
  - key: `node_id`
  - value: `main_token_account_id`
- `WorldState.main_token_treasury_distribution_records: BTreeMap<String, MainTokenTreasuryDistributionRecord>`
  - key: `distribution_id`

### 2) 主链地址绑定
- `World::bind_node_identity(node_id, public_key_hex)` 保持可用，同时补齐主链地址绑定默认派生。
- 新增查询/写接口：
  - `World::node_main_token_account(node_id)`
  - `World::bind_node_main_token_account(node_id, account_id)`
- 桥接分配时，优先使用 `node_main_token_account_bindings[node_id]`。

### 3) 治理绑定（主链策略更新）
- `Action::UpdateMainTokenPolicy { proposal_id, next }` 规则增强：
  - `proposal_id` 必须存在于 `World.proposals`；
  - 对应提案状态必须为 `ProposalStatus::Approved` 或 `ProposalStatus::Applied`。
- 原有参数边界校验和延迟生效规则保持不变。

### 4) treasury 分发闭环
- 新增动作：
```rust
Action::DistributeMainTokenTreasury {
  proposal_id: ProposalId,
  distribution_id: String,
  bucket_id: String,
  distributions: Vec<MainTokenTreasuryDistribution>,
}
```
- 新增事件：
```rust
DomainEvent::MainTokenTreasuryDistributed {
  proposal_id: ProposalId,
  distribution_id: String,
  bucket_id: String,
  total_amount: u64,
  distributions: Vec<MainTokenTreasuryDistribution>,
}
```
- 允许分发 bucket：
  - `staking_reward_pool`
  - `ecosystem_pool`
  - `security_reserve`
- 状态应用：
  - 扣减 treasury bucket；
  - 增加目标账户 `liquid_balance`；
  - 增加 `circulating_supply`（`total_supply` 不变）；
  - 写入 `main_token_treasury_distribution_records`，按 `distribution_id` 幂等防重。

## 里程碑
- M0：设计/项目文档建档。
- M1：节点主链地址绑定接线 + NodePoints 桥接改造。
- M2：主链策略更新治理绑定落地。
- M3：treasury 分发动作/事件/状态闭环落地。
- M4：测试矩阵与文档回写收口。

## 风险
- 兼容性风险：旧快照缺少新增状态字段，需要 `serde(default)` 保证兼容。
- 治理流程风险：若环境中未走 `proposals` 流程，策略更新会被拒绝（需测试覆盖和文档提示）。
- 经济风险：treasury 分发动作如果无治理绑定将存在滥发风险，本期通过 `proposal_id` 绑定降低风险。
- 地址质量风险：当前地址格式为 runtime 内字符串，不等价链外钱包地址。
