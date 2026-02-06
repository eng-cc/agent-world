# Agent World Simulator：LLM 驱动 Agent 行为落地（项目管理文档）

## 任务拆解
- [x] LLM1 输出设计文档（`doc/world-simulator/llm-agent-behavior.md`）
- [x] LLM2 输出项目管理文档（本文件）
- [x] LLM3 新增 `LlmAgentConfig` 与 `config.toml` 加载（含 system prompt 默认值）
- [x] LLM4 新增 OpenAI 兼容 Chat Completions 客户端
- [x] LLM5 新增 `LlmAgentBehavior` 并接入 `AgentBehavior` trait
- [x] LLM6 新增决策协议解析（wait/wait_ticks/move_agent/harvest_radiation）
- [x] LLM7 补充单元测试（配置、解析、失败降级）
- [x] LLM8 更新 README 与 `config.example.toml`（配置说明）
- [x] LLM9 回顾并更新设计文档/项目管理文档状态，补任务日志，运行测试

## 依赖
- `crates/agent_world/src/simulator/agent.rs`
- `crates/agent_world/src/simulator/tests/runner.rs`
- `README.md`
- `config.example.toml`

## 状态
- 当前阶段：LLM9（收尾完成）
- 下一步：按 runtime 路线补齐 LLM effect/receipt 审计闭环
- 最近更新：完成 LLM Agent 行为落地（config.toml + 默认 system prompt + 测试通过，2026-02-06）
