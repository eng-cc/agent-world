# Agent World Simulator：规则 Wasm 化就绪（第二阶段）设计文档

## 目标
- 为规则迁移到 wasm 执行器做最小必要准备：让内核 pre-action 规则可读取只读世界上下文。
- 在不改动既有动作业务语义的前提下，提高规则表达能力（可基于当前世界状态决策）。
- 维持默认无规则时行为完全不变，保持 KRH 基线回归可通过。

## 范围
- In Scope：
  - 扩展 `add_pre_action_rule_hook(...)` 签名，允许 hook 读取 `WorldKernel` 只读视图。
  - 更新 `WorldKernel::step` 规则调用链，向 pre-action hook 传递内核上下文。
  - 补充基于上下文的规则测试（读取时间、读取模型状态）。
  - 保持 deny/modify/allow 合并与执行语义不变。
- Out of Scope：
  - 真正加载和执行 wasm 规则模块。
  - `KernelRuleCost` 扣费生效（本阶段仍只透传/记录）。
  - 增加新的领域动作或资源结算逻辑。

## 接口 / 数据
- pre-action hook 接口由：
  - `Fn(ActionId, &Action) -> KernelRuleDecision`
- 调整为：
  - `Fn(ActionId, &Action, &WorldKernel) -> KernelRuleDecision`
- 语义约束：
  - `&WorldKernel` 为只读上下文，不允许在 hook 内修改世界。
  - 规则仍通过 `KernelRuleDecision` 输出决策，交由统一 merge/execution。

## 里程碑
- M1：完成接口扩展与内核调用链改造。
- M2：完成上下文读取测试与兼容回归。
- M3：更新项目管理文档与 devlog，并提交任务。

## 风险
- Hook 闭包签名变更可能影响现有调用点：通过编译期约束和测试修正覆盖。
- 传入 `&WorldKernel` 可能引入借用复杂度：保持调用阶段只读，避免跨阶段持有引用。
- 上下文可见信息增加导致规则行为漂移：通过 KRH 基线回归测试兜底。
