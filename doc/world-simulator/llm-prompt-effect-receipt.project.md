# Agent World Simulator：LLM Prompt ModuleCall Effect/Receipt 可回放链路（项目管理文档）

## 任务拆解
- [x] LPER1 输出设计文档（`doc/world-simulator/llm-prompt-effect-receipt.md`）
- [x] LPER2 输出项目管理文档（本文件）
- [x] LPER3 扩展 `AgentDecisionTrace`（intent/receipt 结构化字段）
- [x] LPER4 扩展 `WorldEventKind`（LlmEffectQueued/LlmReceiptAppended）
- [x] LPER5 在 `LlmAgentBehavior` 生成 module_call intent/receipt
- [x] LPER6 在 `AgentRunner` 将 trace 事件写入 `kernel.journal`
- [x] LPER7 补充单元测试（链路与序列化）
- [x] LPER8 回顾文档状态、补 devlog、运行测试并收口

## 依赖
- `crates/agent_world/src/simulator/agent.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/runner.rs`
- `crates/agent_world/src/simulator/llm_agent.rs`

## 状态
- 当前阶段：LPER8（已收口，viewer 集成兼容已补齐）
- 下一步：将 simulator 侧 `llm.prompt.module_call` 与 runtime 统一到共享 effect schema
- 最近更新：补齐 agent_world_viewer 对新增 trace/event 字段的兼容并通过全仓测试（2026-02-09）
