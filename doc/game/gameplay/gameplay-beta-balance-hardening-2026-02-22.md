# Gameplay 内测数值加固（治理票权 + 策略授权 + 合约声誉去通胀）

## 目标
- 收敛当前内测前的三类高风险数值/规则问题：
  - 治理投票 `weight` 可被任意放大。
  - `UpdateGameplayPolicy` 缺少治理授权门槛。
  - 经济合约成功结算的声誉收益与真实经济贡献脱钩，存在通胀风险。
- 保持 Runtime 协议兼容与 `test_tier_required/test_tier_full` 测试口径稳定。

## 范围

### In Scope
- Runtime gameplay 动作校验增强：
  - 为 `CastGovernanceVote.weight` 增加上限。
  - 为 `UpdateGameplayPolicy` 增加治理授权校验。
- 经济合约结算数值修正：
  - 将成功结算声誉奖励改为“与结算金额相关 + 受质押上限约束 + 全局上限保护”的去通胀模型。
- 定向测试补齐与已有断言修正。

### Out of Scope
- 新增治理动作类型或链上治理协议。
- 对战争/危机全套数值再平衡。
- Viewer/UI 展示改造。

## 接口/数据
- `Action::CastGovernanceVote`：新增 `weight` 上限规则（`weight <= 100`，超出即拒绝）。
- `Action::UpdateGameplayPolicy`：新增授权规则：
  - 操作方需具备已通过的治理提案历史（`GovernanceProposalStatus::Passed`）；
  - 且该提案 `total_weight_at_finalize >= 3`。
- `Action::SettleEconomicContract(success=true)`：
  - `creator_reputation_delta/counterparty_reputation_delta` 由 `reputation_stake` 直接映射改为：
    - 基于 `settlement_amount` 的基础奖励（`settlement_amount / 10`，最小为 `1`）；
    - 受 `reputation_stake` 和全局奖励上限双重约束；
    - 当前全局奖励上限：`12`；
    - 失败路径保持惩罚逻辑。

## 里程碑
- M0：建档与任务拆解。
- M1：投票权重上限落地并通过回归。
- M2：策略更新治理授权落地并通过回归。
- M3：合约声誉去通胀公式落地并通过回归。
- M4：文档/devlog/项目状态收口。

## 风险
- 权重上限过低可能降低治理表达能力。
  - 缓解：先以保守上限上线，后续基于回放指标调优。
- 策略授权门槛可能影响既有自动化脚本。
  - 缓解：补充测试用治理预热步骤，并给出明确拒绝原因。
- 合约声誉奖励下调可能降低合作动机。
  - 缓解：保留最小正奖励并持续观察治理/合约触发率。
