# Gameplay Module-Driven Production Closure（生产级设计）

## 目标
- 把 Gameplay 层从“runtime 硬编码推进”切换为“WASM gameplay 模块驱动推进”，让战争/治理/危机/元进度具备可替换、可演进实现。
- 将 m5 内建模块从占位实现升级为可执行规则模块，确保输出稳定协议（directive emit）并可由 runtime 消费。
- 补齐 LLM/Simulator 决策协议，使 Agent 能表达并提交战争、政治、危机、元进度动作。
- 完成顶层项目管理文档中与 Gameplay Kernel API / 模块 MVP / 测试矩阵相关未闭环项。

## 范围

### In Scope
- m5 内建模块生产实现：`war/governance/crisis/economic/meta`。
- Gameplay 模块输出协议：统一 `gameplay.lifecycle.directives` emit，按 tick 输出生命周期指令。
- Runtime 模块驱动闭环：在 `step_with_modules` 中消费 Gameplay module emits，生成 DomainEvent。
- 模块订阅调整：m5 模块增加 tick 订阅并保持 post_event 状态同步。
- LLM 决策协议补齐：新增 gameplay actions 的 schema、解析、prompt/action label 对齐。
- 测试补齐：`test_tier_required` 覆盖模块驱动治理/危机/战争闭环和 LLM 新动作解析。

### Out of Scope
- 数值平衡深度调参（仅保持当前可运行 baseline）。
- UI 可视化改版。
- 分布式节点之间的 gameplay 状态同步协议升级。

## 接口/数据

### 模块输出协议
- emit kind：`gameplay.lifecycle.directives`
- payload：
  - `directives[]`，按 `type` 区分：
    - `governance_finalize`
    - `crisis_spawn`
    - `crisis_timeout`
    - `war_conclude`
    - `meta_grant`

### Runtime 消费映射
- `governance_finalize` -> `DomainEvent::GovernanceProposalFinalized`
- `crisis_spawn` -> `DomainEvent::CrisisSpawned`
- `crisis_timeout` -> `DomainEvent::CrisisTimedOut`
- `war_conclude` -> `DomainEvent::WarConcluded`
- `meta_grant` -> `DomainEvent::MetaProgressGranted`

### LLM/Simulator 动作协议扩展
新增 decision/action：
- `form_alliance`
- `declare_war`
- `open_governance_proposal`
- `cast_governance_vote`
- `resolve_crisis`
- `grant_meta_progress`

## 里程碑
- GMPC-R1：m5 模块从占位实现升级为可执行规则逻辑。
- GMPC-R2：`step_with_modules` 改为 Gameplay module-driven，runtime 可消费 directive emits。
- GMPC-R3：LLM/Simulator 决策协议补齐并通过 required-tier 测试。
- GMPC-R4：顶层项目管理文档收口，测试矩阵条目闭环。

## 风险
- 模块状态与 runtime 状态如果口径不一致，可能出现重复结算或漏结算，需要严格用事件驱动状态收敛。
- directive 协议演进时若缺少版本约束，可能导致旧模块输出被错误解析。
- Gameplay 模块 tick 频率提升会增加执行成本，需要通过 wake 策略和输出上限控制成本。
