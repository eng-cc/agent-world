# Agent World Simulator：LLM 用户文本输出与工具调用分区可视化（设计文档）

## 目标
- 改造 LLM 决策流程，使 Agent 能在决策 JSON 中携带面向用户的文本信息，并进入会话消息链路。
- 改造 Viewer Chat 可视化，把“信息流（玩家/Agent/系统）”与“基础工具调用流（module call）”分开展示。
- 工具调用展示尽量直观：明确模块名、状态、参数与结果摘要，避免将原始长 JSON 混入普通聊天气泡。

## 范围

### 范围内
- LLM 输出协议扩展：支持在决策输出中携带可选用户文本字段（如 `message_to_user`）。
- 解析链路扩展：`decision/module_call/execute_until/decision_draft` 均可提取用户文本。
- 行为循环改造：仅在存在用户文本时写入 `LlmChatRole::Agent` 消息；不再把原始 LLM JSON 直接作为 Agent 对话消息。
- Prompt 规则更新：输出 schema 与 hard rules 增加 `message_to_user` 可选字段规范。
- Viewer Chat 面板改造：
  - 信息流面板只显示 `player/agent/system`。
  - 工具调用面板单独显示 `tool`，并按模块调用卡片化渲染。
- 测试补齐：
  - `test_tier_required`：解析器、行为循环消息落盘、Viewer 分区聚合与渲染辅助逻辑。

### 范围外
- 不新增后端持久化存储（仍基于 trace 聚合展示）。
- 不改动 Viewer 协议结构（复用现有 decision trace 字段）。
- 不重构 LLM provider 与网络调用实现。

## 接口 / 数据

### 1) LLM 输出 JSON 扩展
- 在以下输出对象中支持可选字段：
  - 终态决策 JSON
  - `{"type":"module_call", ...}`
  - `{"type":"decision_draft", ...}`
  - `{"decision":"execute_until", ...}`
- 新增可选字段：
  - `message_to_user: string`（trim 后非空时生效）

示例：
- 终态决策：
  - `{"decision":"move_agent","to":"loc-2","message_to_user":"我将前往 loc-2 进行侦察。"}`
- 工具调用：
  - `{"type":"module_call","module":"environment.current_observation","args":{},"message_to_user":"我先读取当前环境信息。"}`

### 2) 会话消息写入策略
- `player`：沿用当前注入逻辑。
- `agent`：仅写入 `message_to_user`。
- `tool`：沿用模块调用反馈，但格式化为可解析结构文本，便于 Viewer 卡片化。
- `system`：沿用动作结果与运行时反馈。

### 3) Viewer 分区展示
- 信息流（Info）：`player/agent/system`。
- 工具流（Tool Calls）：`tool`。
- 工具流卡片字段：
  - 模块名 `module`
  - 状态 `status`
  - 参数摘要 `args`
  - 结果摘要 `result`
  - 时间 `T{time}`

## 里程碑
- M1：设计与项目管理文档完成。
- M2：LLM 输出解析与行为循环接入 `message_to_user`。
- M3：Viewer 信息流/工具流分区与工具调用卡片化完成。
- M4：测试回归、文档回写、devlog 收口。

## 风险
- 模型输出兼容风险：旧输出无 `message_to_user`。
  - 缓解：字段可选，缺失时保持旧决策链路可用。
- 工具结果体积风险：结果过长影响 UI 可读性。
  - 缓解：展示层做摘要截断，保留可复制文本。
- 角色分区误判风险：历史数据格式不统一导致解析失败。
  - 缓解：解析失败降级为原文展示，不阻塞主信息流。
