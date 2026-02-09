# Agent World Simulator：LLM Prompt 组装重构与多步决策机制（项目管理文档）

## 任务拆解
- [x] LMSO1 输出设计文档（`doc/world-simulator/llm-prompt-multi-step-orchestration.md`）
- [x] LMSO2 输出项目管理文档（本文件）
- [ ] LMSO3 抽象 Prompt 组装层（`PromptAssemblyInput/PromptSection/PromptAssemblyOutput`）
- [ ] LMSO4 接入 Prompt 分段预算与历史裁剪（保留协议段，低优先级先裁剪）
- [ ] LMSO5 扩展配置模型与读取（`MAX_DECISION_STEPS/MAX_REPAIR_ROUNDS/PROMPT_MAX_HISTORY_ITEMS/PROMPT_PROFILE`）
- [ ] LMSO6 实现多步状态机（`plan -> module_call* -> decision_draft -> final_decision`）
- [ ] LMSO7 实现 repair 分支（解析失败后的格式修复轮次）
- [ ] LMSO8 扩展 trace（`llm_step_trace/llm_prompt_section_trace`）
- [ ] LMSO9 补充单元测试（组装、流转、上限、兼容）
- [ ] LMSO10 更新 README / config 示例 / 开发日志并收口

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `README.md`
- `config.example.toml`
- `doc/world-simulator/llm-prompt-system.md`

## 状态
- 当前阶段：LMSO2（文档阶段完成）
- 下一步：进入 LMSO3，先拆分 Prompt 组装层，解除硬编码拼接
- 最近更新：完成“Prompt 组装重构 + 多步机制”设计与任务拆解（2026-02-09）
