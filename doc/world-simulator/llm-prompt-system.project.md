# Agent World Simulator：Agent-LLM Prompt 模块交互系统（项目管理文档）

## 任务拆解
- [x] LPS1 输出设计文档（`doc/world-simulator/llm-prompt-system.md`）
- [x] LPS2 输出项目管理文档（本文件）
- [x] LPS3 扩展 `LlmAgentConfig`（短期目标/长期目标/模块调用上限）
- [x] LPS4 新增 Agent 级目标覆盖规则（`<AGENT_ID_NORMALIZED>`）
- [x] LPS5 实现 Prompt 模块协议解析（decision + module_call）
- [x] LPS6 实现内置模块执行器（环境/短期记忆/长期记忆/模块清单）
- [x] LPS7 接入 `AgentMemory` 到 `LlmAgentBehavior` 生命周期
- [x] LPS8 补充单元测试（配置、拼接、模块闭环、超限降级）
- [x] LPS9 更新 README 与 `config.example.toml`
- [x] LPS10 回顾文档状态，补 devlog，跑测试并收口

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/memory.rs`
- `README.md`
- `config.example.toml`

## 状态
- 当前阶段：LPS10（已收口）
- 下一步：按 runtime 路线将模块调用轨迹接入 effect/receipt 可回放链路
- 最近更新：完成 Prompt 模块交互系统落地与测试（2026-02-08）
