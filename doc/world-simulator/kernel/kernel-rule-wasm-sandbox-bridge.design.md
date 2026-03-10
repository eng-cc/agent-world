# 规则 Wasm Sandbox 桥接设计

- 对应需求文档: `doc/world-simulator/kernel/kernel-rule-wasm-sandbox-bridge.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/kernel-rule-wasm-sandbox-bridge.project.md`

## 1. 设计定位
定义模拟器规则模块与 runtime `ModuleSandbox` 的桥接输入、输出与错误处理约定。

## 2. 设计结构
- 输入映射：`KernelRuleModuleInput` 到 runtime `ModuleCallInput`。
- 输出映射：从 `ModuleOutput.emits` 中提取 `rule.decision` 载荷。
- 桥接执行：以单个 pre-action 规则模块为单位调用 `ModuleSandbox`。

## 3. 关键接口 / 入口
- `WorldKernel::set_pre_action_wasm_rule_module_evaluator(...)`
- `ModuleCallInput` / `ModuleOutput.emits` / `rule.decision` payload
- sandbox bridge 测试入口

## 4. 约束与边界
- 无 decision emit 视为 `allow`。
- 多 decision / 非法 payload / action_id 不匹配均视为错误。
- 本阶段不处理模块治理与多模块编排。

## 5. 设计演进计划
- 先保持现有规则语义稳定。
- 再沿项目文档任务拆解推进实现与回归。
