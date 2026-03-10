# 规则 Wasm 执行接线基础设计

- 对应需求文档: `doc/world-simulator/kernel/kernel-rule-wasm-executor-foundation.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/kernel-rule-wasm-executor-foundation.project.md`

## 1. 设计定位
定义模拟器规则模块输入/输出契约、可选 wasm evaluator 接线与错误兜底策略。

## 2. 设计结构
- 输入层：`KernelRuleModuleContext` 与 `KernelRuleModuleInput`。
- 输出层：`KernelRuleModuleOutput` 封装 `KernelRuleDecision`。
- 接线层：在 pre-action 阶段提供可选 wasm 规则评估入口。

## 3. 关键接口 / 入口
- `WorldKernel::set_pre_action_wasm_rule_evaluator(..)`
- `KernelRuleModuleContext` / `KernelRuleModuleInput` / `KernelRuleModuleOutput`
- 错误映射到 `RuleDenied` 的备注协议

## 4. 约束与边界
- 未配置 evaluator 时保持 KWR 阶段行为不变。
- wasm 评估失败不能 silent failure。
- 本阶段不引入真实字节码加载与治理。

## 5. 设计演进计划
- 先保持现有规则语义稳定。
- 再沿项目文档任务拆解推进实现与回归。
