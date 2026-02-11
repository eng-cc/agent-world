# Agent World Simulator：LLM Prompt 组装重构与多步决策机制（项目管理文档）

## 任务拆解
- [x] LMSO1 输出设计文档（`doc/world-simulator/llm-prompt-multi-step-orchestration.md`）
- [x] LMSO2 输出项目管理文档（本文件）
- [x] LMSO2A 补充“上下文长度与记忆平衡”设计（预算模型/记忆打分/裁剪顺序）
- [x] LMSO3 抽象 Prompt 组装层（`PromptAssemblyInput/PromptSection/PromptAssemblyOutput`）
- [x] LMSO4 接入 Prompt 分段预算与历史裁剪（保留协议段，低优先级先裁剪）
- [x] LMSO4A 实现 PromptBudget 估算器（输入上限/输出预留/安全边际）
- [x] LMSO4B 实现 MemorySelector（候选打分/去重/Top-K 打包）
- [x] LMSO5 扩展配置模型与读取（`MAX_DECISION_STEPS/MAX_REPAIR_ROUNDS/PROMPT_MAX_HISTORY_ITEMS/PROMPT_PROFILE`）
- [x] LMSO6 实现多步状态机（`plan -> module_call* -> decision_draft -> final_decision`）
- [x] LMSO7 实现 repair 分支（解析失败后的格式修复轮次）
- [x] LMSO8 扩展 trace（`llm_step_trace/llm_prompt_section_trace`）
- [x] LMSO9 补充单元测试（组装、流转、上限、兼容）
- [x] LMSO10 更新 README / config 示例 / 开发日志并收口
- [x] LMSO11 长跑压测（多步+repair+记忆膨胀）并固化回归测试
- [x] LMSO12 真实 LLM 长跑脚本压测（指标落盘 + 阈值断言 + 结果汇总）
- [x] LMSO13 压测 run.log 增加 LLM 输入输出落盘（逐 tick）
- [x] LMSO14 反重复门控 + execute_until 持续执行动作（防复读并保留连续动作能力）
- [x] LMSO15 execute_until 多事件停止条件 + 输出容错解析（event_any_of / `a|b` / decision_draft 简写）
- [x] LMSO16 真实运行态 LLM I/O 日志体积治理（`--llm-io-max-chars`）
- [x] LMSO17 LLM 请求超时策略调优（默认分钟级超时，短超时自动回退）
- [x] LMSO18 execute_until 的 until.event 扩展语义（能量/热状态/采集阈值事件 + until.value_lte）
- [x] LMSO19 Prompt 上下文收敛（Module History 大结果压缩）+ 压测脚本无 jq 指标回退解析
- [x] LMSO20 OpenAI 兼容 tools 注册与 tool_calls/function_call 解析（保留 module_call 文本协议兼容）
- [x] LMSO21 单轮多段输出顺序消费（同次 completion 解析并消费多个 JSON 片段）
- [x] LMSO22 LLM 请求层迁移至 `async-openai` Responses API（工具注册/调用解析适配）

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/behavior_loop.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/execution_controls.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `config.example.toml`
- `README.md`

## 状态
- 当前阶段：LMSO28（峰值收敛已完成）
- 下一步：进入 LMSO29（稳定性与动作成功率协同优化）
- 最近更新：完成 LMSO28 30 tick 回归并收口（2026-02-11）

## 增量优化任务（30 tick 观察）
- [x] LMSO23 解析噪声收敛（输入侧优先）：强化 prompt 单 JSON 约束、阶段收敛门槛与模块白名单，先降低多段输出/超限 module_call 导致的 parse_error。
- [x] LMSO24 动作参数护栏：为 `harvest_radiation.max_amount` 增加可配置上限与提示词约束，并在运行时统一裁剪超限值。
- [x] LMSO25 决策效率优化：优化 `execute_until` 模板与重入策略，引入连续动作自动重入，降低无效 plan/module_call 轮次。
- [x] LMSO26 Prompt 预算收敛：继续压缩 memory/history，降低 `llm_input_chars_max` 与 `prompt_section_clipped`。
- [x] LMSO27 效率回收：在保持峰值受控下，回收 `module_call` 与 `llm_input_chars_avg/total`。

## 状态（增量）
- 当前增量阶段：LMSO29（待启动）
- 增量目标：在保持峰值收敛成果下，继续优化动作成功率与策略稳定性。
- 最近更新：完成 LMSO28 回归并收敛峰值输入（`llm_input_chars_max: 14216 -> 9373`，2026-02-11）。

## LMSO27 任务进展（2026-02-10）
- [x] LMSO27A 重规划门控收敛：仅对“重复上一动作”的终态 decision 触发强制重规划，允许无 module_call 直接切换到新动作。
- [x] LMSO27B 30 tick 回归对比：确认门控收敛后 `module_call` 与 `llm_input_chars_avg/total` 进一步下降。

## 状态（LMSO27 后）
- 当前增量阶段：LMSO28（已完成）
- 最近更新：LMSO28 已在 2026-02-11 收口并进入 LMSO29 规划。


## LMSO28 任务拆解（2026-02-11）
- [x] LMSO28A 双阈值峰值预算：在 Prompt 预算中增加 soft/hard 峰值目标与分级压缩。
- [x] LMSO28B 高波动源压缩：收敛 observation/history/memory 注入体积，优先压缩软段。
- [x] LMSO28C 30 tick 回归对比：验证 `llm_input_chars_max/avg/total`、`module_call` 与稳定性指标。

## 状态（LMSO28）
- 当前子阶段：LMSO29（进行中，详见 `doc/world-simulator/llm-lmso29-stability.md`）
- 目标：在保持峰值已收敛的基础上，进一步回收动作失败率并稳定行动效率。
- 最近验证：`llm_input_chars_max 14216 -> 9373`，`llm_input_chars_avg 1786 -> 1121`，`module_call 9 -> 2`（`llm_errors=0`、`parse_errors=0`）。
