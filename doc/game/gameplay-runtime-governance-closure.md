# Gameplay Runtime Governance Closure（生产级设计）

## 目标
- 将 `doc/game/gameplay-engineering-architecture.md` 中“Gameplay Layer 只通过受限接口接入运行时”的要求落到可执行代码，而不是停留在文档层。
- 为 War/Governance/Crisis/Economic/Meta 五类玩法模块提供统一、可审计、可治理的生产级接入约束。
- 在不破坏现有世界不变量（资源守恒、时间单向、权限/能力边界）的前提下，引入“玩法模块就绪度”可观测能力，支撑上线前验收。

## 范围

### In Scope
- 扩展 WASM ABI：新增 Gameplay 模块元数据模型。
- 扩展模块角色：新增 `ModuleRole::Gameplay`。
- Runtime 校验增强：
  - Gameplay 角色必须声明合法玩法元数据。
  - 非 Gameplay 角色禁止夹带玩法元数据。
  - 模块变更（register/activate/deactivate/upgrade）后，按 `(game_mode, gameplay_kind)` 检测激活冲突。
- Runtime 查询增强：
  - 输出当前激活玩法模块清单。
  - 输出按 mode 的玩法就绪报告（覆盖度/缺失类型）。
- 配套测试覆盖（runtime 单测 + required 路径回归）。

### Out of Scope
- 战争伤害公式、投票权重、危机生成器等具体玩法规则实现。
- 客户端 UI 展示改造与可视化页面重设计。
- 链上部署脚本、社区模块审核流程。

## 接口/数据

### ABI 新增数据
- `GameplayModuleKind`：`war | governance | crisis | economic | meta`
- `GameplayContract`：
  - `kind`
  - `game_modes`
  - `min_players`
  - `max_players`
- `ModuleRole` 新增 `Gameplay`。
- `ModuleAbiContract` 新增可选字段 `gameplay`。

### Runtime 新增能力
- 模块清单接口：
  - `World::gameplay_modules() -> Vec<ActiveGameplayModule>`
- 模式就绪度接口：
  - `World::gameplay_mode_readiness(mode) -> GameplayModeReadiness`
  - 输出 `coverage`、`missing_kinds`、`is_ready()`

### 治理校验策略
- 角色与元数据一致性：
  - `role=Gameplay` 必须有 `abi_contract.gameplay`
  - 非 Gameplay 角色必须没有 `abi_contract.gameplay`
- 元数据字段校验：
  - `game_modes` 非空，且每项非空、无重复
  - `min_players >= 1`
  - `max_players` 若存在，必须 `>= min_players`
- 激活冲突校验：
  - 同一 `game_mode` 下，同一 `GameplayModuleKind` 同时只能有一个激活模块。

## 里程碑
- GRC-1：ABI 与 Runtime 基础校验接入（角色/元数据一致性 + 字段合法性）。
- GRC-2：模块治理变更后的玩法槽位冲突检测（mode+kind 唯一）。
- GRC-3：玩法模块清单与模式就绪报告接口 + 单测回归。

## 风险
- 旧模块若误标为 Gameplay 但未补元数据，会在 shadow 阶段被拒绝，需要迁移窗口。
- 多团队并行开发模块时，`mode+kind` 槽位冲突会增多，需要发布前统一编排。
- 如果后续要支持同 kind 多模块并行（分区/权重路由），需在本模型上扩展路由规则而非放开约束。
