# Agent World Simulator：规则 Wasm 执行接线基础（第三阶段）设计文档

## 目标
- 在不改变当前规则语义（deny/modify/allow merge）的前提下，为模拟器内核接入 wasm 规则执行器建立最小接线层。
- 定义稳定的“规则模块输入/输出”数据契约，便于后续把真实 wasm sandbox 调用替换进来。
- 保持默认路径行为不变：未配置 wasm 规则执行器时，内核行为应与 KWR 阶段一致。

## 范围
- In Scope：
  - 新增模拟器规则 wasm 评估输入/输出结构（可序列化、可测试）。
  - 在 `WorldKernel::step` pre-action 阶段增加可选 wasm 规则评估入口（默认关闭）。
  - 定义错误兜底策略：wasm 评估失败时转换为 `RuleDenied` 备注，避免 silent failure。
  - 增加最小闭环测试（allow/deny/modify 与错误路径）。
- Out of Scope：
  - 真实 wasm 字节码加载、缓存、沙箱限制和 host function 实现。
  - 规则 `cost` 扣费落账生效（仍维持透传/记录）。
  - 规则模块生命周期治理（发布、升级、回滚）与分布式同步。

## 接口 / 数据
- 新增输入结构（草案）：
  - `KernelRuleModuleContext`：包含时间与最小世界快照（如 location/agent 索引）。
  - `KernelRuleModuleInput`：`action_id` + `action` + `context`。
- 新增输出结构（草案）：
  - `KernelRuleModuleOutput`：封装 `KernelRuleDecision`。
- 新增可选评估入口：
  - `WorldKernel::set_pre_action_wasm_rule_evaluator(...)`
  - 评估器签名接收 `&KernelRuleModuleInput`，返回 `Result<KernelRuleModuleOutput, String>`。
- 执行策略：
  - wasm 评估输出作为一个规则决策源，与现有 pre-hook 决策统一 merge。
  - 评估失败转换为 deny 决策（带错误说明），保障系统可解释。

## 里程碑
- M1：完成 KWE-1（数据契约 + 可选评估入口 + 默认路径不变）。
- M2：完成 KWE-2（接线测试：allow/deny/modify/错误路径）。
- M3：完成 KWE-3（回归验证与文档收口）。

## 风险
- 输入上下文过大导致后续 wasm 调用开销偏高：先用最小上下文字段，后续按需扩展。
- 评估器错误处理不当会影响动作可用性：统一转为 `RuleDenied` 并携带错误说明。
- 新接线可能引入行为漂移：通过 KWR 既有回归与新增接线测试双重兜底。
