# Agent World Simulator：Agent-LLM Prompt 模块交互系统（设计文档）

## 目标
- 构建一套可扩展的 Agent-LLM Prompt 交互机制，让 LLM 不只“看见观测”，还能通过机器人模块按需查询上下文。
- 支持 LLM 通过模块获取：
  - 环境信息（当前观测）
  - 记忆信息（短期记忆 + 长期记忆，类人脑分层）
  - Agent 可用模块清单（知道“自己有什么能力可调”）
- 为每个 Agent 增加外部可配置目标：短期目标、长期目标，并将其拼接进 system prompt。
- 保持稳定性：模块调用协议异常、解析失败、调用超限时都安全降级为 `Wait`。

## 范围

### In Scope
- 扩展 `LlmAgentConfig`，增加目标配置与模块调用轮次限制。
- 在 `LlmAgentBehavior` 内新增 Prompt 模块交互会话：
  - LLM 输出 `module_call` 请求
  - 系统执行模块并回填结果
  - LLM 基于结果继续决策
- 将 `AgentMemory` 实际接入 LLM 行为循环，形成可查询的短期/长期记忆。
- system prompt 模板化拼接：基础 prompt + 短期目标 + 长期目标 + 模块协议。
- 补充单元测试（配置读取、目标拼接、模块调用闭环、超限降级）。

### Out of Scope
- 引入真实 function calling/tool calling 协议（本期维持 JSON 协议）。
- 向 runtime effect/receipt 持久化完整模块调用轨迹。
- 语义检索（embedding）与向量数据库记忆检索。
- 多模型路由、成本预算、复杂重试编排。

## 接口 / 数据

### 配置项（`config.toml` 或同名环境变量）
- 已有：
  - `AGENT_WORLD_LLM_MODEL`
  - `AGENT_WORLD_LLM_BASE_URL`
  - `AGENT_WORLD_LLM_API_KEY`
  - `AGENT_WORLD_LLM_TIMEOUT_MS`
  - `AGENT_WORLD_LLM_SYSTEM_PROMPT`
- 新增：
  - `AGENT_WORLD_LLM_SHORT_TERM_GOAL`
  - `AGENT_WORLD_LLM_LONG_TERM_GOAL`
  - `AGENT_WORLD_LLM_MAX_MODULE_CALLS`（每次决策最大模块调用轮次）
- Agent 级覆盖（可选）：
  - `AGENT_WORLD_LLM_SHORT_TERM_GOAL_<AGENT_ID_NORMALIZED>`
  - `AGENT_WORLD_LLM_LONG_TERM_GOAL_<AGENT_ID_NORMALIZED>`
  - 其中 `AGENT_ID_NORMALIZED` 为大写+非字母数字转下划线（如 `agent-1` → `AGENT_1`）。

### Prompt 交互协议（JSON）
- 模块调用：
  - `{"type":"module_call","module":"memory.short_term.recent","args":{"limit":5}}`
- 最终决策：
  - `{"decision":"wait"}`
  - `{"decision":"wait_ticks","ticks":3}`
  - `{"decision":"move_agent","to":"loc-2"}`
  - `{"decision":"harvest_radiation","max_amount":20}`

### 机器人模块（内置）
- `agent.modules.list`
  - 返回模块清单、用途、参数约定。
- `environment.current_observation`
  - 返回当前 Observation（当前 tick 的环境快照）。
- `memory.short_term.recent`
  - 返回短期记忆最近 N 条。
- `memory.long_term.search`
  - 按关键词检索长期记忆（无关键词时返回高重要度记忆）。

### 记忆接入策略
- `decide` 前记录本轮 observation 摘要到短期记忆。
- `decide` 后记录决策到短期记忆。
- `on_action_result` 记录动作执行结果；失败动作会进入长期记忆索引，便于后续检索。

## 里程碑
- M1：完成文档与配置模型扩展（目标项 + 模块调用上限）。
- M2：完成 Prompt 模块交互主流程（module_call → 模块结果 → 决策）。
- M3：完成记忆接入与测试，更新 README / 示例配置 / 项目状态。

## 风险
- **输出协议漂移**：LLM 输出非协议 JSON；通过严格解析与降级缓解。
- **调用轮次膨胀**：LLM反复调用模块导致开销上升；通过 `max_module_calls` 限制。
- **记忆噪声累积**：长期记忆质量下降；本期仅提供基础检索，后续可引入摘要压缩与重要度重评分。
- **配置复杂性提升**：Agent 级覆盖键较多；通过“全局默认 + 可选覆盖”降低接入门槛。
