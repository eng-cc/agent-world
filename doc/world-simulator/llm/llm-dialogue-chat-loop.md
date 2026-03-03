# Agent World Simulator：LLM 对话轮次驱动与右侧 Chat 面板（设计文档）

## 目标
- 将当前 LLM Agent 从“step 概念驱动”收敛为“会话轮次驱动”，避免在 prompt/trace 中暴露 `step_index/step_type` 语义。
- 支持稳定多轮对话：
  - 动作被拒绝后，把拒绝原因作为会话反馈提供给 LLM。
  - 工具调用后，把工具结果作为会话反馈提供给 LLM。
- 在 viewer 右侧新增玩家与 Agent 的 Chat 面板，支持玩家向指定 Agent 发送消息并可视化消息流。
- 保持现有 `world_viewer_live --llm` 与 web 闭环链路可用。

## 范围

### In Scope
- LLM 决策循环改造为“轮次”模型（assistant 输出 -> 可能 tool_call -> tool_result 回灌 -> assistant 继续）。
- Prompt 组装移除 step 元信息段，增加会话历史段。
- 新增会话消息模型（玩家、Agent、工具、系统反馈）并落到 `AgentDecisionTrace`，用于 viewer 展示。
- 动作执行结果（成功/拒绝）写入会话历史，并在后续轮次可见。
- viewer 协议新增 Agent Chat 请求/响应。
- viewer 右侧模块新增 Chat 区块（Agent 选择、玩家输入、消息列表）。
- 测试补充：
  - `test_tier_required`：解析、会话回灌、协议序列化、UI 关键逻辑。
  - `test_tier_full`：viewer live + web 闭环回归（保持口径）。

### Out of Scope
- 新模型 provider 切换与成本优化。
- 长期记忆策略重构（只做会话层接入，不做全量记忆系统重写）。
- viewer 视觉风格大改（仅新增功能模块）。

## 接口 / 数据

### 1) LLM 决策轮次
- 新语义：`max_dialogue_turns`（可兼容旧 `max_decision_steps` 配置读取）。
- 每轮只要求返回一个终态决策 JSON 或一个 `module_call` JSON。
- 不再要求/强调 `plan/decision_draft/final_decision` 分阶段输出。
- 若返回 `module_call`：
  - 执行工具；
  - 将工具结果写入会话（tool role）；
  - 继续下一轮。
- 若返回终态决策：结束本 tick 决策。

### 2) 会话消息模型
- 新增 `LlmChatMessageTrace`：
  - `time`
  - `agent_id`
  - `role`（`player | agent | tool | system`）
  - `content`
- `AgentDecisionTrace` 新增：`llm_chat_messages: Vec<LlmChatMessageTrace>`（记录本次决策新增消息）。
- 行动反馈回灌：
  - `on_action_result` 将成功/拒绝事件转为 `system` 消息。
  - 拒绝时明确携带 `RejectReason` 文本。

### 3) Viewer 协议
- `ViewerRequest` 新增：`AgentChat { request }`
  - `request.agent_id`
  - `request.message`
  - `request.player_id`（可选）
- `ViewerResponse` 新增：
  - `AgentChatAck { ack }`
  - `AgentChatError { error }`
- live server 在 `--llm` 模式下把玩家消息注入对应 Agent 会话；脚本模式返回错误。

### 4) Viewer 右侧 Chat 面板
- 右侧模块开关新增 `chat`。
- Chat 区块能力：
  - 选择目标 Agent。
  - 发送玩家消息（通过 `ViewerRequest::AgentChat`）。
  - 展示玩家/Agent/工具/系统消息流（基于 decision trace 聚合）。

## 里程碑
- M1：文档与任务拆解完成。
- M2：LLM 会话轮次驱动 + 拒绝/工具反馈回灌完成。
- M3：viewer 协议与右侧 Chat 面板完成。
- M4：测试与文档回写收口。

## 风险
- 会话长度膨胀风险：通过消息条数与字符摘要上限裁剪。
- 协议兼容风险：新增字段保持向后兼容（旧字段保留）。
- UI 交互风险：发送失败需可见错误反馈，避免“静默失败”。
