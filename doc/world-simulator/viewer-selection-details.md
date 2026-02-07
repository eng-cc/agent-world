# Viewer 选中对象详情面板（含 LLM 决策 I/O）

## 目标
- 在 viewer 中新增“选中对象详情”面板：点击 Agent 或 Location 后显示尽量详细的信息。
- 对 Agent 详情补充 LLM 决策调试信息，至少包含最近决策的 LLM 输入（prompt）与输出（completion/错误）。
- 保持离线回放兼容：无 LLM trace 数据时仍可展示基础详情并明确提示“无 LLM trace”。
- 遵循可视化总原则：通过详情面板以最直接的方式获取对象相关的模拟信息。

## 范围
- **范围内**
  - `agent_world_viewer` 新增详情 UI 区块与文本渲染逻辑。
  - `viewer` 协议新增决策 trace 消息（用于 live 模式）。
  - `LlmAgentBehavior` 暴露最近一次决策 trace（输入/输出/解析错误）。
  - `world_viewer_live` 在 LLM 驱动模式下推送决策 trace 给 viewer。
  - 新增/更新单元测试，覆盖协议 round-trip、live trace 推送、详情文案渲染。
- **范围外**
  - 不实现历史 trace 分页/检索系统。
  - 不为非 LLM agent 伪造 LLM I/O（仅显示“非 LLM 或无 trace”）。

## 接口 / 数据
- 新增数据结构：`AgentDecisionTrace`
  - 字段：`agent_id`、`time`、`decision`、`llm_input`、`llm_output`、`llm_error`
- 协议扩展：`ViewerResponse::DecisionTrace { trace }`
- 详情面板展示策略：
  - Agent：基础状态（位置、坐标、资源、电力/热状态）+ 最近相关事件 + 最近 LLM trace
  - Location：基础状态（名称、坐标、资源、profile 摘要、碎片预算摘要）+ 最近相关事件

## 里程碑
- **M1**：完成文档与任务拆解。
- **M2**：打通 LLM trace 采集与 live 协议下发。
- **M3**：完成 viewer 详情面板渲染与测试覆盖。

## 风险
- **信息量过大**：详情文本可能过长，需控制展示窗口（保留最近 N 条事件/trace）。
- **模式差异**：离线与 script 模式没有 LLM trace，UI 需要明确降级文案。
- **兼容性**：协议新增字段需保持向后兼容（未知消息不影响已有逻辑）。
