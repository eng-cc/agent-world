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

## 4. 约束与边界
- 创世分配总和必须为 `10000 bps`。
- 项目战略控制目标为 `5000 bps`，单人直接受益硬上限为 `1500 bps`。
- 创世液态流通硬上限为 `500 bps`。
- 早期奖励只能按贡献证据发放，不能用 `play-to-earn` 叙事替代。

## 5. 设计演进计划
- 先冻结比例和控制边界。
- 再落实具体 bucket/account/vesting 参数表与审计 checklist。
- 最后根据是否需要 fully on-chain 分发，决定 early contributor reserve 的长期执行载体。

