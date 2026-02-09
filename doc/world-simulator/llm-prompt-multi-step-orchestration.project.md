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
- 当前阶段：LMSO18（until.event 扩展语义已完成）
- 下一步：基于真实运行态继续扩充停止条件（例如更细粒度的资源/库存事件）并观察误触发率
- 最近更新：完成 LMSO18（新增 insufficient_electricity / thermal_overload / harvest_yield_below / harvest_available_below，并引入 `until.value_lte` 约束，2026-02-09）
