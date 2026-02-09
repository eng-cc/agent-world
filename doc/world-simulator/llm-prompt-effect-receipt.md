# Agent World Simulator：LLM Prompt ModuleCall Effect/Receipt 可回放链路（设计文档）

## 目标
- 将 LLM `module_call` 交互轨迹从“仅调试文本”升级为结构化的 `effect/receipt` 事件链。
- 让每次模块调用在 simulator `journal` 中可审计、可回放、可复盘。
- 与 runtime 的 `EffectQueued / ReceiptAppended` 概念保持语义一致，降低后续对齐成本。

## 范围

### In Scope
- 在 `AgentDecisionTrace` 增加结构化字段：`llm_effect_intents`、`llm_effect_receipts`。
- 在 LLM 决策循环中为每次 `module_call` 生成 intent + receipt 记录。
- 在 `AgentRunner::tick` 将上述记录写入 `WorldKernel.journal`（新增事件类型）。
- 补充测试：解析、链路写入、序列化与兼容性。

### Out of Scope
- 复用 runtime 的签名/策略校验逻辑（本次仅做 simulator 侧链路）。
- 将 effect/receipt 写入 runtime 分布式索引与哈希验证。
- 引入真实外部执行器（本期模块调用仍为本地内置模块）。

## 接口 / 数据

### 新增结构（Simulator）
- `LlmEffectIntentTrace`
  - `intent_id`
  - `kind`（固定 `llm.prompt.module_call`）
  - `params`（`module/args`）
  - `cap_ref`（固定 `llm.prompt.module_access`）
  - `origin`（固定 `llm_agent`）
- `LlmEffectReceiptTrace`
  - `intent_id`
  - `status`（`ok` / `error`）
  - `payload`（模块返回结果）
  - `cost_cents`（预留，当前 `None`）

### 事件扩展（`WorldEventKind`）
- `LlmEffectQueued { agent_id, intent }`
- `LlmReceiptAppended { agent_id, receipt }`

### 写入时机
1. `LlmAgentBehavior` 在处理 `module_call` 时生成 intent 与 receipt。
2. `AgentRunner::tick` 读取 `decision_trace` 中的 intent/receipt。
3. 逐条写入 `kernel.journal`，确保随 world 快照/日志回放。

## 里程碑
- M1：定义结构体与事件类型。
- M2：接入 LLM 决策循环与 runner 入链逻辑。
- M3：补充测试并更新文档/日志。

## 风险
- **事件膨胀**：高频模块调用会增加日志体积；通过 `max_module_calls` 上限缓解。
- **语义漂移**：与 runtime effect 字段不一致；通过固定字段名和 kind 常量降低偏差。
- **兼容性**：旧 trace 无新字段；通过 `serde(default)` 保证向后兼容。
