# Agent World Simulator：规则 Wasm Sandbox 桥接（第四阶段）设计文档

## 目标
- 将模拟器内核已有的可选 wasm pre-action evaluator，与 runtime 的 `ModuleSandbox` 调用链打通。
- 在保持默认行为不变的前提下，提供“可落地的真实沙箱调用路径”，替代手写闭包模拟。
- 保持失败可解释：沙箱调用失败或输出非法时，统一转为 `RuleDenied` 备注，不出现 silent failure。

## 范围
- In Scope：
  - 在 `WorldKernel` 增加“基于 `ModuleSandbox` 安装 pre-action wasm evaluator”的桥接 API。
  - 定义 simulator -> runtime sandbox 的最小输入映射（`KernelRuleModuleInput` -> `ModuleCallInput`）。
  - 解析 `ModuleOutput.emits` 中 `rule.decision` 载荷并转换为 `KernelRuleDecision`。
  - 增加桥接回归测试（请求编码、allow/deny/modify、调用失败兜底）。
- Out of Scope：
  - 模块治理（注册/升级/分发）与 artifact 存储。
  - 多模块订阅编排（本阶段只桥接单个 pre-action 规则模块调用）。
  - `KernelRuleCost` 账本扣费生效。

## 接口 / 数据
- 新增桥接 API（草案）：
  - `WorldKernel::set_pre_action_wasm_rule_module_evaluator(...)`
  - 入参包含模块标识（`module_id/wasm_hash/entrypoint/limits/wasm_bytes`）与 `ModuleSandbox` 实例。
- 输入映射约定：
  - 组装 runtime `ModuleCallInput`（`ctx/action`），其中 `action` 携带序列化后的 `KernelRuleModuleInput`。
  - `ctx.origin.kind` 使用 `simulator_action`，`ctx.origin.id` 为 `action_id`。
- 输出映射约定：
  - 读取 `ModuleOutput.emits` 中 `kind=rule.decision` 的 payload，反序列化为 `KernelRuleDecision`。
  - 无 decision emit 视为 `allow`；多 decision / 非法 payload / action_id 不匹配视为错误。

## 里程碑
- M1：完成 KWS-1（桥接 API 与输入输出转换）。
- M2：完成 KWS-2（桥接测试闭环）。
- M3：完成 KWS-3（回归验证与文档收口）。

## 风险
- simulator 与 runtime 产生接口耦合：通过最小桥接 API 控制边界，避免引入治理逻辑。
- 序列化口径不一致导致模块不可读：测试覆盖 CBOR 编解码和 payload 校验。
- 沙箱锁争用或故障影响吞吐：本阶段先保证正确性，后续再评估并发优化。
