# Agent World Simulator：LLM 驱动 Agent 行为落地（设计文档）

## 目标
- 在现有 `AgentBehavior` 抽象上落地一套可运行的 **LLM Agent 行为实现**，用于替代纯规则型 `decide` 逻辑。
- 采用 **OpenAI 兼容 API** 完成推理调用，支持以 `config.toml` 注入关键配置。
- 新增 `system prompt` 配置能力；当未配置时，默认使用：
  - `硅基个体存在的意义是保障硅基文明存续和发展；`
- 保持运行时稳定性：当 LLM 调用失败或输出不可解析时，Agent 应安全降级为 `Wait`，避免破坏模拟闭环。

## 范围

### In Scope
- 新增 `LlmAgentBehavior`，实现 `AgentBehavior` trait。
- 新增 `LlmAgentConfig`（模型、端点、鉴权、超时、system prompt）。
- 新增 system prompt 配置读取与默认值回退。
- 新增 OpenAI 兼容 Chat Completions 客户端（同步调用）。
- 新增 LLM 输出到 `AgentDecision` 的最小解析协议（`wait / wait_ticks / move_agent / harvest_radiation`）。
- 新增单元测试（配置读取、默认 prompt、决策解析、失败降级）。

### Out of Scope
- 预算与成本计费（token/cost）完整治理链路。
- 记忆模块（Memory Module）WASM 化落地。
- 在 runtime effect/receipt 流中持久化 LLM 输入输出（本次仅在 simulator 侧落地最小闭环）。
- 多模型路由、缓存与重试策略编排。

## 接口 / 数据

### 配置文件项（`config.toml`）
- `AGENT_WORLD_LLM_MODEL`
- `AGENT_WORLD_LLM_BASE_URL`
- `AGENT_WORLD_LLM_API_KEY`
- `AGENT_WORLD_LLM_TIMEOUT_MS`（可选，默认 30_000）
- `AGENT_WORLD_LLM_SYSTEM_PROMPT`
  - 可选；缺省时使用默认 system prompt：
    - `硅基个体存在的意义是保障硅基文明存续和发展；`

说明：键名沿用 `AGENT_WORLD_*` 前缀以保持兼容语义，但项目约定通过 `config.toml` 作为主配置入口。

### 配置加载优先级
1. 若项目根目录存在 `config.toml`，优先读取该文件。
2. 若 `config.toml` 不存在，则回退读取进程环境变量。
3. 在读取 `config.toml` 时，若单个键不存在，允许回退到同名环境变量。

### 核心结构
- `LlmAgentConfig`
  - `model`, `base_url`, `api_key`, `timeout_ms`, `system_prompt`
- `LlmCompletionClient` trait
  - `complete(request) -> Result<String, LlmClientError>`
- `OpenAiChatCompletionClient`
  - 对接 OpenAI 兼容 `/chat/completions`
- `LlmAgentBehavior<C: LlmCompletionClient>`
  - 在 `decide` 中调用 LLM 并解析为 `AgentDecision`

### Demo 入口（已落地）
- `world_llm_agent_demo`
  - 启动后为场景内每个 agent 构造 `LlmAgentBehavior::from_env(agent_id)`。
  - 使用 `AgentRunner` 执行 `observe -> decide -> act` 循环。
  - 支持参数：`--scenario <name>`、`--ticks <n>`；默认场景为 `llm_bootstrap`。

### 决策协议（LLM 输出）
- 约定输出 JSON：
  - `{"decision":"wait"}`
  - `{"decision":"wait_ticks","ticks":3}`
  - `{"decision":"move_agent","to":"loc-2"}`
  - `{"decision":"harvest_radiation","max_amount":20}`
- 若输出不合法或字段缺失，回退 `AgentDecision::Wait`。

## 里程碑
- M1：完成 `LlmAgentConfig` 与 system prompt 默认回退。
- M2：完成 OpenAI 兼容客户端与 `LlmAgentBehavior` 主流程。
- M3：补充测试与文档（README / project / devlog）。

## 风险
- **输出不稳定风险**：LLM 可能输出非 JSON；通过“严格协议 + 解析失败降级”缓解。
- **网络依赖风险**：在线调用可能超时或失败；通过超时配置与 `Wait` 降级缓解。
- **安全风险**：错误 prompt 可能诱导越权动作；当前通过动作白名单与解析协议收敛风险。
- **一致性风险**：未接入 receipt 体系前，跨运行不可严格重放；后续在 runtime effect/receipt 中补齐。
