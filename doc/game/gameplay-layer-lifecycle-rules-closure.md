# Gameplay Layer Lifecycle Rules Closure（生产级设计）

## 目标
- 在已完成“基础层 / WASM 游戏层拆分”和“玩法原语补齐”的基础上，补齐可持续运行的玩法生命周期规则。
- 让战争、治理、危机、元进度从“单次动作”提升为“可推进、可收敛、可审计”的完整回路。
- 保持世界不变量不被玩法层绕过：资源守恒、时间单向、Agent 唯一身份、事件可追溯。

## 范围

### In Scope
- 新增治理提案生命周期：开启提案、投票约束、超时收敛。
- 新增危机生命周期：周期生成、可响应、超时失败。
- 新增战争生命周期：宣战后自动结算与归档。
- 增强元进度：按轨道积分自动解锁阶段成就。
- 新增 gameplay tick 循环处理器，统一在 step 周期推进上述生命周期。
- 新增 `test_tier_required` 覆盖治理/危机/战争/元进度闭环。

### Out of Scope
- 复杂平衡参数调优（如胜率、经济惩罚曲线、随机扰动策略）。
- UI 展示和可视化面板改版。
- 跨节点治理同步协议改造。

## 接口/数据

### 新增 Action 原语
- `OpenGovernanceProposal`：开启治理提案并声明投票窗口、选项、通过阈值。

### 新增 DomainEvent 原语
- `GovernanceProposalOpened`
- `GovernanceProposalFinalized`
- `CrisisSpawned`
- `CrisisTimedOut`
- `WarConcluded`

### 状态模型扩展
- `governance_proposals: BTreeMap<String, GovernanceProposalState>`
- `CrisisState` 扩展为生命周期状态（Active/Resolved/TimedOut）。
- `WarState` 扩展结算字段（结束时刻、胜方、结算摘要）。
- `MetaProgressState` 增加按轨道阶段解锁记录。

### Tick 推进约束（MVP）
- 危机按固定周期生成（可配置常量）。
- 危机到期未处理则自动失败并写入超时事件。
- 治理提案到期自动结算（法定人数 + 通过阈值）。
- 战争在持续时间窗口结束后自动结算并归档。

## 里程碑
- GLC2-R1：协议与状态模型扩展（治理提案/危机/战争生命周期事件）。
- GLC2-R2：gameplay tick 生命周期推进器落地并接入 `step`/`step_with_modules`。
- GLC2-R3：required-tier 闭环测试通过，文档与 devlog 收口。

## 风险
- 生命周期自动推进会增加每 tick 处理成本，需要保持 O(n) 可控并避免热点扫描退化。
- 治理提案与投票状态不一致时可能导致收敛偏差，需要严格在 `Action -> Event -> State` 闭环内验证。
- 危机自动生成频率过高会导致策略噪声，需要后续结合玩法数据调参。
- 战争自动结算规则如果过于简单，可能出现策略单一化，后续需补平衡测试。
