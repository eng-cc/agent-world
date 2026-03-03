# Agent World Simulator：LLM 请求层迁移至 async-openai Responses API（设计文档）

## 目标
- 将现有 OpenAI 兼容 `chat/completions` 请求层替换为 `async-openai` SDK，并统一走 Responses API。
- 保持现有 `LlmAgentBehavior` 多步决策协议稳定：`plan -> module_call* -> decision_draft -> final decision`。
- 保留并适配工具注册与调用链路：内置模块能力继续以工具方式暴露，并映射回内部 `module_call` 协议。
- 保留超时回退策略（短超时首发 + 默认长超时重试），降低线上慢响应导致的误降级。

## 范围

### In Scope
- 在 `crates/agent_world` 引入 `async-openai`（Responses API）依赖并完成客户端替换。
- 将请求构造从 `messages + tools/tool_choice` 迁移为 Responses 的 `instructions + input + tools`。
- 将响应解析从 `choices.message.content/tool_calls/function_call` 迁移为 Responses `output`（`function_call` / 文本输出）。
- 适配并回归测试工具注册与函数参数解析逻辑（含参数 JSON 解析与 module 名映射）。
- 更新 README、配置说明和相关项目文档状态。

### Out of Scope
- 不改动 LLM 决策协议本身（`decision_flow` 的协议字段保持兼容）。
- 不引入流式 Responses（本期仍采用非流式同步调用）。
- 不改动业务动作与世界模拟规则。

## 接口 / 数据

### 1) LLM 客户端接口
- 保持 `LlmCompletionClient` trait 不变：
  - 输入：`LlmCompletionRequest { model, system_prompt, user_prompt }`
  - 输出：`LlmCompletionResult { output, model, prompt_tokens, completion_tokens, total_tokens }`
- 内部实现改为：
  - `CreateResponseArgs` + `client.responses().create(...)`
  - `instructions <- system_prompt`
  - `input <- user_prompt`

### 2) 工具注册
- 使用 Responses API `Tool::Function(FunctionTool)` 注册 4 个模块工具：
  - `agent_modules_list` -> `agent.modules.list`
  - `environment_current_observation` -> `environment.current_observation`
  - `memory_short_term_recent` -> `memory.short_term.recent`
  - `memory_long_term_search` -> `memory.long_term.search`
- 参数 schema 延续现有 JSON Schema 结构，保持与模块执行器一致。

### 3) 工具调用解析
- 优先从 Responses `output` 中提取 `OutputItem::FunctionCall`。
- 将函数调用统一映射为内部 JSON：
  - `{"type":"module_call","module":"...","args":{...}}`
- 若无函数调用则回退读取 `output_text()`，继续沿用现有文本协议解析链路。

### 4) 超时与端点策略
- 端点基准 URL 兼容旧配置：
  - 允许输入 `/v1`、`/v1/`、`/v1/chat/completions`、`/v1/responses`，统一归一到 API Base（例如 `/v1`）。
- 超时策略保持：
  - 配置超时 `< 默认超时` 时，超时后自动使用默认超时重试一次。

## 里程碑
- M1 文档与方案确认：完成本设计文档与项目管理文档。
- M2 代码迁移：客户端替换到 `async-openai` Responses API。
- M3 工具链路适配：工具注册、函数调用解析、参数映射完成。
- M4 回归与文档收口：单测、`cargo check`、README/配置更新、开发日志更新。

## 风险
- **SDK 类型变更风险**：`async-openai` Responses 类型字段可能升级变更；通过聚焦官方类型与本地单测兜底。
- **兼容端点风险**：历史配置可能仍填 `chat/completions` 路径；通过 base URL 归一化兼容。
- **超时行为差异风险**：从 blocking reqwest 到 async client 可能引入错误类型差异；统一映射到 `LlmClientError` 并补回归测试。

## 已知问题与后续优化点（2026-02-10）

基于实跑命令
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo -- llm_bootstrap --ticks 30 --report-json .tmp/llm_multi_round_30/report.json`
的结果（`active_ticks=30`、`llm_errors=0`、`parse_errors=4`）：

1. **Responses 兼容输出仍有解析噪声**
   - 现象：30 tick 内出现 `parse_errors=4`，虽被 repair 机制兜底但仍增加了降级风险。
   - 优化方向：继续扩展“宽松解析”覆盖面（多段 JSON/嵌套文本/字段缺省变体），并补充真实 provider 样本回归集。

2. **连续动作参数出现异常放大**
   - 现象：`harvest_radiation.max_amount` 在运行中出现极大值（如 `999999999`）。
   - 优化方向：在动作层增加参数护栏（clamp/阈值配置），并在 prompt 协议中显式限制动作参数范围。

3. **决策效率仍有可优化空间**
   - 现象：`decision_wait=4` 且 `world_time=26 < active_ticks=30`，存在空转 tick。
   - 优化方向：强化 `execute_until` 终止条件模板与降级后重入策略，减少无效 `Wait`。

4. **Prompt 上下文仍存在膨胀与裁剪**
   - 现象：`llm_input_chars_max=18175`、`prompt_section_clipped=8`。
   - 优化方向：继续优化 memory digest 压缩策略（按类别限额、重复动作折叠、摘要优先级重排）。
