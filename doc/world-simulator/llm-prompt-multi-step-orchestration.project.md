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

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `README.md`
- `config.example.toml`
- `doc/world-simulator/llm-prompt-system.md`

## 状态
- 当前阶段：LMSO11（长跑压测已固化）
- 下一步：补充真实场景脚本压测（viewer_live + llm_bootstrap）与指标落盘
- 最近更新：完成 LMSO11 压测回归（240 tick、每 tick 5 次LLM阶段循环，2026-02-09）
