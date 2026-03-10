# 内核不变量回归与规则 Hook 基座设计

- 对应需求文档: `doc/world-simulator/kernel/kernel-rule-hook-foundation.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/kernel-rule-hook-foundation.project.md`

## 1. 设计定位
定义模拟器内核的 pre_action / post_action 规则 Hook 基座、决策结构与默认 no-op 路径。

## 2. 设计结构
- 回归层：先固定当前动作语义作为基线。
- Hook 层：在 `WorldKernel::step` 提供 pre/post action 可插拔调用点。
- 决策层：统一 `allow/deny/modify` 与 override merge 规则。

## 3. 关键接口 / 入口
- `add_pre_action_rule_hook(...)` / `add_post_action_rule_hook(...)`
- `KernelRuleDecision` / `KernelRuleVerdict` / `KernelRuleCost`
- `WorldKernel::step` 的 hook 调用顺序

## 4. 约束与边界
- 默认未注册 Hook 时行为必须与现状一致。
- `deny` 优先于 `modify/allow`。
- 多个 `modify` 需输出一致 override，否则拒绝。

## 5. 设计演进计划
- 先保持现有规则语义稳定。
- 再沿项目文档任务拆解推进实现与回归。
