# oasis7 主链 Token 创世参数正式执行清单（2026-03-22）

审计轮次: 1

## Meta
- Owner Role: `runtime_engineer`
- Review Roles: `producer_system_designer`, `qa_engineer`
- Source Topic: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.prd.md`
- Freeze Status: `logic_frozen_address_binding_pending`
- Runtime Anchor:
  - `Action::InitializeMainTokenGenesis`
  - `MainTokenGenesisAllocationPlan`
  - `MainTokenGenesisAllocationBucketState`
  - recipient `vested_balance`

## 1. Freeze Gates
- Gate-1: `main_token_config.initial_supply` 必须在 mint 前冻结为单一绝对值；本清单当前只冻结比例和算法，不伪造最终 absolute amount。
- Gate-2: 所有 bucket `ratio_bps` 总和必须保持 `10000`。
- Gate-3: 所有 bucket `genesis_liquid` 继续固定为 `0`；创世后只允许通过 `ClaimMainTokenVesting` 转成 liquid。
- Gate-4: 所有 `recipient_account_id` / multisig 地址在真实 mint 前必须从 slot 升级为具体链上账户，否则 QA 结论最多为 `conditional_draft_only`。
- Gate-5: 执行金额必须使用 runtime 真值 rounding 规则，不允许手工改表补差额。

## 2. Allocated Amount Rule
- Step-1: 对每个 bucket 先计算 `floor(initial_supply * ratio_bps / 10000)`。
- Step-2: 将 remainder 按 `ratio_bps` 降序、`bucket_id` 升序逐个 `+1` 分配，直到 remainder 清零。
- Step-3: 最终 `sum(allocated_amount)` 必须严格等于 `initial_supply`。
- Step-4: event 应用后，每个 recipient 对应账户的 `vested_balance` 为其名下所有 bucket 的 `allocated_amount` 聚合值。

## 3. Slot Registry
| slot_id | expected_object | current_value | freeze_requirement | status |
| --- | --- | --- | --- | --- |
| `acct.team_core_vesting.v1` | team core vesting recipient account | `TBD_BEFORE_MINT` | 必须绑定真实 recipient account | `pending_binding` |
| `acct.early_contributor_reward.v1` | reward reserve recipient account | `TBD_BEFORE_MINT` | 必须绑定真实 reward reserve account | `pending_binding` |
| `acct.node_service_custody.v1` | node service custody account | `TBD_BEFORE_MINT` | 必须绑定真实 custody account | `pending_binding` |
| `acct.staking_custody.v1` | staking custody account | `TBD_BEFORE_MINT` | 必须绑定真实 custody account | `pending_binding` |
| `acct.ecosystem_governance.v1` | ecosystem governance account | `TBD_BEFORE_MINT` | 必须绑定真实 governance account | `pending_binding` |
| `acct.security_council_reserve.v1` | security reserve account | `TBD_BEFORE_MINT` | 必须绑定真实 security reserve account | `pending_binding` |
| `acct.foundation_ops.v1` | ops reserve account | `TBD_BEFORE_MINT` | 必须绑定真实 ops reserve account | `pending_binding` |
| `msig.team_core.v1` | team multisig / beneficiary controller | `TBD_BEFORE_MINT` | 必须冻结签名门限与 signer list | `pending_binding` |
| `msig.reward_reserve.v1` | reward reserve multisig | `TBD_BEFORE_MINT` | 必须冻结签名门限与审批链 | `pending_binding` |
| `msig.node_committee.v1` | node service committee | `TBD_BEFORE_MINT` | 必须冻结 signer list / governance hook | `pending_binding` |
| `msig.staking_governance.v1` | staking governance multisig | `TBD_BEFORE_MINT` | 必须冻结 signer list / governance hook | `pending_binding` |
| `msig.ecosystem_governance.v1` | ecosystem governance multisig | `TBD_BEFORE_MINT` | 必须冻结 signer list / cadence | `pending_binding` |
| `msig.security_council.v1` | security council multisig | `TBD_BEFORE_MINT` | 必须冻结 signer list / emergency rule | `pending_binding` |
| `msig.foundation_ops.v1` | ops multisig | `TBD_BEFORE_MINT` | 必须冻结 signer list / budget cadence | `pending_binding` |

## 4. Bucket Execution Sheet
| bucket_id | ratio_bps | recipient_slot_id | recipient_account_id | controller_slot_id | signer_policy | start_epoch | cliff_epochs | linear_unlock_epochs | claim_cadence | runtime_target | freeze_status | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `team_long_term_vesting` | `2000` | `acct.team_core_vesting.v1` | `TBD_BEFORE_MINT` | `msig.team_core.v1` | team multisig + beneficiary split table | `0` | `365` | `1095` | quarterly | `MainTokenGenesisAllocationPlan -> bucket state -> team recipient vested_balance` | `ready_pending_address_binding` | 个人受益拆分表必须单列并接受 `<=1500 bps` 审计 |
| `early_contributor_reward_reserve` | `1500` | `acct.early_contributor_reward.v1` | `TBD_BEFORE_MINT` | `msig.reward_reserve.v1` | reward multisig + producer approval | `0` | `0` | `3650` | batch-by-batch only | `MainTokenGenesisAllocationPlan -> bucket state -> reward reserve vested_balance` | `ready_pending_address_binding` | limited preview 期间保持独立 reward reserve，不并入 `ecosystem_pool` |
| `node_service_genesis_custody` | `2000` | `acct.node_service_custody.v1` | `TBD_BEFORE_MINT` | `msig.node_committee.v1` | protocol governance / node committee | `0` | `180` | `1825` | governance batch | `MainTokenGenesisAllocationPlan -> bucket state -> node service custody vested_balance` | `ready_pending_address_binding` | 这是 custody account，不是 `node_service_reward_pool` |
| `staking_genesis_custody` | `1500` | `acct.staking_custody.v1` | `TBD_BEFORE_MINT` | `msig.staking_governance.v1` | protocol governance | `0` | `180` | `1825` | governance batch | `MainTokenGenesisAllocationPlan -> bucket state -> staking custody vested_balance` | `ready_pending_address_binding` | 这是 custody account，不是 `staking_reward_pool` |
| `ecosystem_governance_reserve` | `1500` | `acct.ecosystem_governance.v1` | `TBD_BEFORE_MINT` | `msig.ecosystem_governance.v1` | ecosystem governance multisig | `0` | `90` | `1460` | quarterly | `MainTokenGenesisAllocationPlan -> bucket state -> ecosystem governance vested_balance` | `ready_pending_address_binding` | 不等于公开营销池 |
| `security_reserve_emergency` | `1000` | `acct.security_council_reserve.v1` | `TBD_BEFORE_MINT` | `msig.security_council.v1` | security council emergency policy | `0` | `0` | `0` | emergency only | `MainTokenGenesisAllocationPlan -> bucket state -> security reserve vested_balance` | `ready_pending_address_binding` | 常态不 claim，仅事故或防御动作使用 |
| `foundation_ops_reserve` | `500` | `acct.foundation_ops.v1` | `TBD_BEFORE_MINT` | `msig.foundation_ops.v1` | ops multisig budget policy | `0` | `90` | `730` | monthly or quarterly | `MainTokenGenesisAllocationPlan -> bucket state -> ops reserve vested_balance` | `ready_pending_address_binding` | 运营与基础设施盘 |

## 5. Pre-Mint Checklist
- [ ] 冻结 `main_token_config.initial_supply`
- [ ] 绑定全部 `recipient_account_id`
- [ ] 绑定全部 `controller_slot_id` 对应真实 multisig / governance account
- [ ] 输出创始人个人受益拆分表，并证明任一自然人直接受益 `<=1500 bps`
- [ ] 用本清单 + `token-genesis-allocation-audit-template-2026-03-22.md` 跑 QA 审计
- [ ] 获得最终 QA `pass`
- [ ] 确认执行 payload 使用 runtime rounding 规则，而不是人工 spreadsheet 改写

## 6. Execution Order
1. 冻结 `initial_supply`
2. 绑定 slot registry 到真实链上账户
3. 依据 runtime rounding 规则生成 7 条 `MainTokenGenesisAllocationPlan`
4. 交 QA 填正式 audit template
5. QA `pass` 后再准备 `InitializeMainTokenGenesis`
6. 创世后抽查 recipient `vested_balance` 与 bucket `allocated_amount`

## 7. Not Ready Conditions
- 任何 `recipient_account_id = TBD_BEFORE_MINT`
- 任何 controller multisig 未冻结 signer rule
- 创始人个人受益拆分表缺失
- `initial_supply` 仍未冻结
- QA 仍为 `conditional_draft_only` 或 `block`
