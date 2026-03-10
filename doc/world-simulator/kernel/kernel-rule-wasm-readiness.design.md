# 规则 Wasm 化就绪（第二阶段）设计

- 对应需求文档: `doc/world-simulator/kernel/kernel-rule-wasm-readiness.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/kernel-rule-wasm-readiness.project.md`

## 1. 设计定位
定义规则系统在真正接入 wasm evaluator 之前的最小就绪层：允许 pre-action hook 读取 `WorldKernel` 只读上下文，但不改变现有规则决策与动作语义。

## 2. 设计结构
- Hook 接口层：将 pre-action hook 扩展为可接收 `&WorldKernel` 的只读上下文签名。
- 调用链路层：`WorldKernel::step` 在规则执行前统一注入内核只读视图。
- 决策合并层：继续复用现有 `KernelRuleDecision` merge/execute 规则，不引入新的 verdict 类型。
- 回归验证层：以时间、模型状态等只读观测场景验证上下文读取能力。

## 3. 关键接口 / 入口
- `add_pre_action_rule_hook(...)`
- `Fn(ActionId, &Action, &WorldKernel) -> KernelRuleDecision`
- `WorldKernel::step`
- `KernelRuleDecision` / `KernelRuleCost`

## 4. 约束与边界
- `&WorldKernel` 仅用于只读查询，hook 内不得修改世界状态。
- 未注册规则时必须保持 KRH 基线行为完全一致。
- 本阶段不加载 wasm 模块，不使 `KernelRuleCost` 实际扣费。
- 借用生命周期必须限制在单次规则评估窗口内，避免跨阶段持有引用。

## 5. 设计演进计划
- 先完成 hook 签名扩展与 step 调用链改造。
- 再补上下文读取测试，确认默认行为不漂移。
- 最后为后续 wasm evaluator 接线保留稳定输入契约。
