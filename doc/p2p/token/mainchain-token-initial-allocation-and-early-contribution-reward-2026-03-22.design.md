# oasis7 主链 Token 初始分配与早期贡献奖励口径（2026-03-22）设计

- 对应需求文档: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.prd.md`
- 对应项目管理文档: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.project.md`

## 1. 设计定位
定义主链 Token 创世分配、控制边界、低流通门禁与早期贡献奖励的统一设计，让 Token 发行从一开始就是“可审计的战略配置”，而不是“事后补解释的营销行为”。

## 2. 设计结构
- 创世分配层：冻结 `10000 bps` 分配表、bucket 命名、控制主体与锁仓方式。
- 控制边界层：区分项目战略控制、协议奖励池、单人直接受益和外部流通。
- 奖励执行层：把 early contributor reward 约束为 evidence-based 审核与受控发放。
- 审计门禁层：对比例求和、个人上限、低流通和禁语边界做 QA/治理复核。

## 3. 关键接口 / 入口
- `Action::InitializeMainTokenGenesis`
- `Action::ClaimMainTokenVesting`
- `Action::DistributeMainTokenTreasury`
- `WorldState.main_token_genesis_buckets`
- `WorldState.main_token_treasury_balances`

## 3.1 TIGR-1 创世参数表（草案）
- 实现约束：
  - `InitializeMainTokenGenesis` 当前只会把 bucket 分配写入 recipient account 的 `vested_balance`，不会直接初始化 `main_token_treasury_balances`。
  - 因此本表中的 `protocol:*` recipient 全部是 custody account 名称草案，不等于 runtime treasury bucket 名称。
  - 当前暂按 `genesis_epoch=0`、`1 epoch ~= 1 day` 进行锁仓换算；若最终链上 epoch 节奏不同，需保持相同自然时间重新换算。

| bucket_id | ratio_bps | recipient | controller | start_epoch | cliff_epochs | linear_unlock_epochs | recommended_claim_cadence | 说明 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `team_long_term_vesting` | `2000` | `protocol:team-core-vesting` | team multisig | `0` | `365` | `1095` | quarterly | 核心团队主锁仓盘 |
| `early_contributor_reward_reserve` | `1500` | `protocol:early-contributor-reward` | reward multisig + producer approval | `0` | `0` | `3650` | batch-by-batch only | 早期贡献奖励储备，控制首年最大可 claim 容量 |
| `node_service_genesis_custody` | `2000` | `protocol:node-service-custody` | protocol governance / node committee | `0` | `180` | `1825` | governance batch | 创世节点服务储备，和 post-genesis `node_service_reward_pool` 区分 |
| `staking_genesis_custody` | `1500` | `protocol:staking-custody` | protocol governance | `0` | `180` | `1825` | governance batch | 创世质押储备，和 post-genesis `staking_reward_pool` 区分 |
| `ecosystem_governance_reserve` | `1500` | `protocol:ecosystem-governance` | ecosystem governance multisig | `0` | `90` | `1460` | quarterly | grant / ecosystem 计划储备 |
| `security_reserve_emergency` | `1000` | `protocol:security-council-reserve` | security council multisig | `0` | `0` | `0` | emergency only | 安全事故与协议防御储备 |
| `foundation_ops_reserve` | `500` | `protocol:foundation-ops` | ops multisig | `0` | `90` | `730` | monthly or quarterly | 基础设施与运营成本储备 |

## 3.2 控制边界汇总
- 项目战略控制：`team_long_term_vesting + early_contributor_reward_reserve + security_reserve_emergency + foundation_ops_reserve = 5000 bps`
- 协议长期储备：`node_service_genesis_custody + staking_genesis_custody = 3500 bps`
- 生态/治理储备：`ecosystem_governance_reserve = 1500 bps`
- 创始人个人直持：不单列独立大 bucket；如需个人受益，必须内嵌在 `team_long_term_vesting` 受益人表内并继续受 `500~1000 bps` 目标区间与 `1500 bps` 硬上限约束。

## 4. 约束与边界
- 创世分配总和必须为 `10000 bps`。
- 项目战略控制目标为 `5000 bps`，单人直接受益硬上限为 `1500 bps`。
- 创世液态流通硬上限为 `500 bps`。
- 早期奖励只能按贡献证据发放，不能用 `play-to-earn` 叙事替代。
- `TIGR-1` 参数表所有 bucket 的 `genesis_liquid` 默认都为 `0`；若后续要形成 liquid，必须通过 claim 动作显式发生并留下审计记录。

## 5. 设计演进计划
- 先冻结比例和控制边界。
- 再落实具体 bucket/account/vesting 参数表与审计 checklist。
- 最后根据是否需要 fully on-chain 分发，决定 early contributor reserve 的长期执行载体。
