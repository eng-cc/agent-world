# M4 资源产业链可玩性优先强化（2026-02-28）

## 目标
- 以“可玩性优先”为原则，优先消除当前资源产业链的断链点和低决策密度问题。
- 在保持现有 runtime 兼容前提下，提升玩家在物流调度、阶段推进、长期经营上的策略空间。
- 将本轮改动约束为可直接进入 `test_tier_required/test_tier_full` 的可验证增量。

## 范围

### In Scope
- 补齐 `polymer_resin` 内置配方链路，移除关键链路对预置库存的强依赖。
- 落地 `ProductProfile.unlock_stage` 阶段门槛校验（当前阶段不足时拒绝排产）。
- 为 `TransferMaterial` 提供显式优先级输入（保留旧推断逻辑兜底）。
- 落地 `ProductProfile.maintenance_sink` 的持续消耗逻辑并对高阶产物给出首版参数。

### Out of Scope
- 复杂订单簿/撮合市场。
- 跨服经济结算与链上治理协议扩展。
- Viewer 大规模交互重构。

## 接口 / 数据

### 1) 配方链路补齐
- 新增内置 recipe 模块：`m4.recipe.smelter.polymer_resin`。
- 新增 `recipe.smelter.polymer_resin` 到 bootstrap 模块清单与 recipe profile。

### 2) 产品阶段门槛
- 在 `ScheduleRecipe` 时对 `plan.produce` 对应 `ProductProfile.unlock_stage` 做校验。
- 当产品门槛高于当前 `industry_progress.stage` 时拒绝动作并输出拒绝原因。

### 3) 物流显式优先级
- 扩展 `Action::TransferMaterial`：新增可选 `priority`。
- 解析顺序：显式 `priority` > `MaterialProfile.default_priority` > 关键词推断。
- 保持默认兼容：不传 `priority` 时行为与旧版本一致。

### 4) 维护消耗接线
- 在 `ScheduleRecipe` 组装实际消耗时叠加 `maintenance_sink`：
  - 来源：`ProductProfile.maintenance_sink`
  - 作用对象：`plan.produce`
  - 合并策略：按材料种类聚合后与 `plan.consume` 一并校验/扣减。
- 首版参数：仅对中后期产物启用非零 sink，避免新手期开局硬阻断。

## 里程碑
- M0：设计/项目管理文档建档。
- M1：补齐 `polymer_resin` 配方链路 + bootstrap/artifact 同步。
- M2：`unlock_stage` + `TransferMaterial.priority` 接线。
- M3：`maintenance_sink` 接线与默认参数调优。
- M4：required/full 回归、文档收口与 devlog。

## 风险
- 平衡风险：维护消耗过高可能导致中期停摆。
- 兼容风险：`TransferMaterial` 动作结构扩展可能影响旧序列化输入。
- 回归风险：新增消耗后会改变既有测试中的库存快照断言。

缓解：
- 采用“可选字段 + 默认值 + 兜底逻辑”。
- 对高阶产物先保守配置 sink，逐步加压。
- 先补 targeted 单测，再跑 required/full 回归。
