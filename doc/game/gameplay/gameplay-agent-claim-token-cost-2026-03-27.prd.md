# Gameplay Agent 认领代币成本与维护机制（2026-03-27）

- 对应设计文档: `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.design.md`
- 对应项目管理文档: `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 当前 agent 认领缺少正式玩法经济约束，容易出现零成本囤积、多号抢占和“拿了不管”的闲置占坑；这既破坏 `Agent 唯一性` 的世界表达，也让主链 token 缺少一个与中循环组织能力直接相连的真实 sink。
- Proposed Solution: 新增 `PRD-GAME-011`，把“认领 agent”定义为一条受控的 gameplay 经济链路：每次认领都必须支付非零 `activation fee`、锁定 `claim bond`、承担 `upkeep`，并配套 `release cooldown`、欠费宽限、强制回收、闲置回收和声誉分层上限。
- Success Criteria:
  - SC-1: 第 1 个 agent 认领也不存在免费路径；所有成功认领都必须满足 `activation_fee_amount > 0`、`claim_bond_amount > 0`、`upkeep_per_epoch > 0`。
  - SC-2: 单账号可同时认领的 agent 数在 v1 受 `reputation_tier` 限制为 `1/2/3` 三档，且 `slot-2/slot-3` 的总成本分别至少为 `slot-1` 的 `1.5x/2.0x`。
  - SC-3: 任一 agent 在同一时刻只能有 1 个正式 `claim_owner_id`；并发争抢、重复认领和无成本续占的误放过率为 `0`。
  - SC-4: 欠费 claim 最迟在 `2` 个 epoch 宽限后进入强制释放；持续闲置 claim 最迟在 `10` 个 epoch 内回收到未认领池。
  - SC-5: `activation fee`、`upkeep`、`bond refund/slash` 必须全部形成可审计事件，并能进入现有 main token 源汇审计链路。

## 2. User Experience & Functionality
- User Personas:
  - 中循环玩家/组织经营者：需要用明确代价换取 agent 控制权，而不是靠抢占或挂机囤位。
  - `producer_system_designer`: 需要把 agent 认领从“模糊权限”收成可平衡、可审计的经济规则。
  - `runtime_engineer`: 需要一套可确定执行的 claim / upkeep / reclaim 状态机与记账规则。
  - `viewer_engineer`: 需要在 UI / pure API 中向玩家清楚展示认领成本、宽限、冷却和回收风险。
  - `qa_engineer`: 需要验证并发争抢、欠费、闲置、多号囤积和经济审计没有旁路。
- User Scenarios & Frequency:
  - 首次建立组织能力时：每个认真进入中循环的玩家至少 1 次。
  - 扩展更多 agent 槽位时：随着玩家声誉提升，多次发生。
  - 日常持有期：每个 upkeep 结算 epoch 都会发生。
  - 释放 / 被回收时：主动退场、欠费或闲置时发生。
- User Stories:
  - PRD-GAME-011: As a 中循环玩家, I want every agent claim to cost main token and require ongoing upkeep, so that agent control reflects actual commitment instead of zero-cost squatting.
  - PRD-GAME-011A: As a `producer_system_designer`, I want the first claim to also be non-free, so that the world does not silently create a “starter free slot” that weakens the sink and encourages alt abuse.
  - PRD-GAME-011B: As a `qa_engineer`, I want forced reclaim and refund/slash outcomes to be deterministic, so that we can test abuse resistance instead of relying on manual moderation.
- Critical User Flows:
  1. Flow-AGC-001: `玩家选择未认领 agent -> 系统返回 slot quote（activation fee / bond / upkeep / cap）-> 玩家确认 -> 扣除 activation fee、锁定 bond -> agent 进入 claimed_active`
  2. Flow-AGC-002: `每个 upkeep epoch 到达 -> 系统尝试结算 upkeep -> 余额足够则继续持有 -> 不足则进入 upkeep_grace`
  3. Flow-AGC-003: `玩家在 cooldown 后主动 release -> 系统结清欠费 -> 退还 bond 剩余部分 -> agent 回到 unclaimed`
  4. Flow-AGC-004: `claim 进入 grace 后仍未补足 upkeep 或连续闲置达到阈值 -> 系统执行 forced_reclaim -> 计算 slash / refund -> agent 回到 unclaimed`
  5. Flow-AGC-005: `玩家尝试认领第 2/3 个 agent -> 系统按 reputation tier 与 slot multiplier 重新报价 -> 超 cap 则拒绝`
- Functional Specification Matrix:

| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Claim Quote | `agent_id`、`claimer_id`、`claim_slot_index`、`reputation_tier`、`claim_cap`、`activation_fee_amount`、`claim_bond_amount`、`upkeep_per_epoch`、`release_cooldown_epochs` | 玩家查看未认领 agent 时返回成本报价与风险提示 | `unclaimed -> quote_ready` | `slot-1 multiplier=1.0`、`slot-2=1.5`、`slot-3=2.0`；`total_upfront_cost = activation_fee_amount + claim_bond_amount` | 仅当 agent 未被认领、玩家未超 cap、且 liquid main token 足够时可确认 |
| Claim Activation | `claim_owner_id`、`claim_started_epoch`、`next_upkeep_epoch`、`claim_bond_locked_amount`、`activation_fee_burn_amount`、`activation_fee_treasury_amount` | 点击确认认领后扣费并锁定 bond | `quote_ready -> claimed_active` | `activation_fee_split_bps = 5000 burn / 5000 treasury`；首个认领也必须扣费 | 同一 agent 同时只能成功 1 个 claim；并发失败方必须原子回滚 |
| Upkeep Settlement | `upkeep_due_epoch`、`upkeep_per_epoch`、`upkeep_paid_amount`、`grace_deadline_epoch`、`delinquent_amount` | 到达结算 epoch 时自动尝试扣除 upkeep | `claimed_active -> claimed_active` 或 `claimed_active -> upkeep_grace` | `upkeep_split_bps = 3000 burn / 7000 treasury`；每次只结算 1 个 epoch 的应付额 | 仅系统结算；owner 可通过补足 liquid balance 恢复 |
| Voluntary Release | `release_requested_epoch`、`release_effective_epoch`、`bond_refund_amount`、`cooldown_satisfied` | owner 主动放弃当前 claim | `claimed_active/upkeep_grace -> released -> unclaimed` | `release_cooldown_epochs = 3`；退款金额 = `claim_bond_locked_amount - unpaid_upkeep - penalties` | 只有当前 owner 可 release；未过 cooldown 不允许 |
| Forced Reclaim | `forced_reason`、`forced_reclaim_epoch`、`forced_penalty_amount`、`bond_refund_amount` | 欠费超宽限或持续闲置时系统回收 | `upkeep_grace/inactive_reclaim_candidate -> forced_reclaimed -> unclaimed` | 欠费宽限 `grace_epochs = 2`；闲置阈值 `7` 个 epoch，最晚 `10` 个 epoch 完成回收；`forced_reclaim_penalty_bps = 2000`（作用于剩余 bond） | 仅系统可执行；owner 不能在最终回收点之后阻断 |
| Reputation Cap | `reputation_tier`、`claim_cap`、`owned_agent_count` | 认领前校验当前账号可占有的 agent 上限 | `eligible -> eligible/blocked_by_cap` | `tier-0 cap=1`、`tier-1 cap=2`、`tier-2+ cap=3` | 非法 tier 或超 cap 直接拒绝，不允许只靠余额绕过 |

- Acceptance Criteria:
  - AC-1: 首个 agent 认领没有任何免费分支；v1 必须显式校验 `activation_fee_amount > 0`、`claim_bond_amount > 0`、`upkeep_per_epoch > 0`。
  - AC-2: 认领成功后必须立即形成 `activation fee` 记账、`bond locked` 状态和下一次 upkeep 结算 epoch，不允许“先占坑后补票”。
  - AC-3: 同一 agent 被两个账号并发认领时，只允许一个成功；失败方不得丢失 token、不得产生脏 claim。
  - AC-4: `slot-2` 和 `slot-3` 的总持有成本必须高于 `slot-1`，且受 `reputation_tier` 上限限制，不能只靠 token 余额无限扩槽。
  - AC-5: owner 主动 release 时，若已过 cooldown 且无未结债务，必须退回剩余 bond，并把 agent 恢复为可认领状态。
  - AC-6: 欠费 claim 在 `2` 个 epoch 宽限后必须被回收；持续闲置 claim 必须在 `10` 个 epoch 内被回收到未认领池。
  - AC-7: 强制回收必须给出 `forced_reason`、`forced_penalty_amount`、`bond_refund_amount`，并通过统一事件与审计字段可追溯。
  - AC-8: Viewer / pure API 必须同时展示 `claim_slot_index`、`activation_fee_amount`、`claim_bond_amount`、`upkeep_per_epoch`、`grace_deadline_epoch`、`release_cooldown_epochs`，不允许 UI/API 各算一套。
  - AC-9: 本机制不得被表述为现实货币付费解锁、公开售卖 agent 或永久产权出售；它是 gameplay 内部的 main token 承诺成本机制。
- Non-Goals:
  - 不在本专题内定义现实货币购买、法币结算或站外商城。
  - 不把 agent 认领做成永久不可回收的链上产权 NFT。
  - 不在本轮引入代理拍卖行、agent 二级交易市场或跨玩家租赁市场。
  - 不在本轮为 claim 成本拍死绝对 token 数值；v1 先冻结公式、状态机和不可突破的边界。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不新增 AI 模型能力，仅涉及 gameplay 规则与状态机）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview:
  - gameplay 层新增 `agent claim economy` 规则：负责报价、claim 状态机、slot multiplier、cap 和回收逻辑。
  - runtime 负责原子扣费、bond 锁定、epoch upkeep 结算、slash / refund 和记账事件。
  - viewer / pure API 只读取 canonical claim 状态与报价字段，不自行推导隐藏成本。
  - main token 账本继续作为唯一价值来源；claim 机制只消费 `liquid main token`，不旁路 signed action / audit 链路。
- Integration Points:
  - `doc/game/prd.md`
  - `doc/game/project.md`
  - `doc/game/gameplay/gameplay-engineering-architecture.md`
  - `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
  - `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md`
  - `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 余额不足：报价可见，但确认认领必须拒绝，并明确缺少的是 `activation fee`、`bond` 还是两者都不足。
  - 并发争抢：两个提交同时命中同一 `agent_id` 时，只允许第一个写入 `claim_owner_id`；第二个返回冲突，不得重复扣费。
  - upkeep 结算时余额不足：进入 `upkeep_grace`，并写出 `grace_deadline_epoch`；宽限内补足余额后恢复 `claimed_active`。
  - release 请求早于 cooldown：拒绝 release，但必须返回剩余 `cooldown_epochs_remaining`。
  - force reclaim 与 owner 同 epoch 操作冲突：以先完成的合法状态迁移为准，后到请求必须基于最新状态重试。
  - 闲置判断：若 `7` 个连续 epoch 无 owner 发起的有效控制动作或 agent 产出的有效推进事件，则进入 `inactive_reclaim_candidate`；若到 `10` 个 epoch 仍未恢复，则强制回收。
  - slash 后 refund 为负：退款下限固定为 `0`，不得从系统额外倒贴。
  - tier 异常：无效 `reputation_tier` 一律按最低 tier 处理，不允许读空后默认开放更多槽位。
  - UI/API 语义缺口：若某端未显示 canonical claim 成本与倒计时，则该端不得宣称支持正式 agent claim 管理。
- Non-Functional Requirements:
  - NFR-AGC-1: 首个 claim 免费路径命中次数必须为 `0`。
  - NFR-AGC-2: 同一 `agent_id` 的并发 claim 误放过率必须为 `0`。
  - NFR-AGC-3: `activation fee`、`upkeep`、`bond refund/slash` 事件覆盖率必须为 `100%`。
  - NFR-AGC-4: viewer / pure API 在 canonical claim 字段上的一致性必须为 `100%`。
  - NFR-AGC-5: 宽限到强制回收的检测延迟 P95 必须 `<= 1 epoch`。
  - NFR-AGC-6: 单账号 agent cap 默认不得超过 `3`，除非后续新 PRD 明确升级。
  - NFR-AGC-7: v1 的成本曲线必须保持单调不降，不允许出现“第 2 个 agent 比第 1 个更便宜”的参数。
  - NFR-AGC-8: 所有 claim 相关 token 变动必须能进入现有经济源汇审计，不得生成审计盲区。
- Security & Privacy:
  - claim / release / upkeep 结算不得绕过主链 token 的签名与审计路径。
  - 不在公开 UI 中暴露与 claim 无关的账户私密资产信息；只展示本次认领所需的必要成本和状态。
  - 强制回收必须基于可重放的状态与事件，不允许人工后台静默改 owner。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 冻结 `activation fee + claim bond + upkeep + release cooldown + tier cap` 的规则口径与状态机。
  - v1.1: 落地 `idle reclaim`、canonical viewer / pure API 展示和 QA abuse suite。
  - v2.0: 基于真实 claim 数据评估 `slot multiplier`、`grace_epochs`、`penalty_bps` 是否需要新一轮调参专题。
- Technical Risks:
  - 风险-1: 如果 claim 成本只锁 bond 不产生真实 sink，囤位问题可能仍然偏轻处罚。
  - 风险-2: 如果 upkeep 过高，会让 agent 控制变成纯惩罚，削弱组织扩张乐趣。
  - 风险-3: 如果 viewer 不清楚展示倒计时与宽限，玩家会把强制回收理解为 bug 而不是规则。
  - 风险-4: 如果 tier cap 只靠离线判断、不进 runtime，alt 账号仍可能形成事实囤积。

## 6. Validation & Decision Record
- Test Plan & Traceability:

| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-GAME-011 | `TASK-GAME-039` | `test_tier_required` | 文档治理检查、根入口/索引/任务映射核验 | agent claim 经济专题挂载 |
| PRD-GAME-011 | `TASK-GAME-040` | `test_tier_required` + `test_tier_full` | claim / upkeep / release / forced reclaim 状态机回归、经济审计对账、并发争抢测试 | runtime 规则执行、token 记账、安全边界 |
| PRD-GAME-011 | `TASK-GAME-041` | `test_tier_required` | Viewer / pure API canonical 字段、报价展示、宽限/冷却倒计时回归 | 玩家表达层、UI/API 一致性 |
| PRD-GAME-011 | `TASK-GAME-042` | `test_tier_required` + `test_tier_full` | abuse suite、长稳回收、经济告警与不变量复核 | QA 守门、反囤积、经济审计 |
| PRD-GAME-011 | `TASK-GAME-043` | `test_tier_required` | producer 平衡复盘、调参边界与继续/回退决策回写 | 版本平衡、后续节奏裁决 |

- Decision Log:

| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-AGC-001 | 第 1 个 agent 认领也必须付费 + 锁 bond + 付 upkeep | 给首个 agent 免费资格，只对第二个起收费 | 免费首槽会直接成为 alt abuse 和零成本囤位入口，且削弱 token sink。 |
| DEC-AGC-002 | 采用 `activation fee + claim bond + upkeep` 三段式 | 只收一次性买断费 | 买断只解决入口，不解决长期占坑与闲置问题。 |
| DEC-AGC-003 | 用 `reputation_tier` 冻结 `1/2/3` 槽位上限，并让后续槽位更贵 | 只靠余额决定能占多少 agent | 只看余额会把组织控制权过度让渡给高资产账号。 |
| DEC-AGC-004 | 强制回收采用“欠费宽限 + 闲置回收”双触发 | 只在玩家手动 release 时释放 agent | 没有系统回收，agent 池会被长期冻结。 |
| DEC-AGC-005 | 先冻结状态机和不可突破边界，绝对价格留给后续平衡调参 | 现在就硬拍绝对 token 数值并直接宣称最终价格 | 当前阶段更缺结构性规则，而不是营销式定价数字。 |
