# Agent World Simulator：Chat Panel 预设区扩展 Agent Prompt 字段编辑（设计文档）

## 目标
- 在现有最右侧 Chat Panel 的“预设 Prompt”区域内，直接提供 `system prompt`、`短期目标`、`长期目标` 的可编辑能力。
- 保持“一个入口完成聊天预设与 Agent Prompt 配置编辑”，避免回到独立 Prompt Ops 面板。
- 继续复用现有 `prompt_control.apply` 协议，不新增后端接口。

## 范围

### 范围内
- Chat Panel 折叠区内新增 Agent Prompt Draft 子区：
  - 按当前选中 Agent 展示并编辑 `system_prompt_override`。
  - 按当前选中 Agent 展示并编辑 `short_term_goal_override`。
  - 按当前选中 Agent 展示并编辑 `long_term_goal_override`。
- 增加“加载当前配置”“应用到 Agent”按钮：
  - 加载：从当前 viewer 状态中的最新 profile 读取字段。
  - 应用：发送 `ViewerRequest::PromptControl::Apply`。
- 与已有“聊天预设编辑/填充输入框”共用同一折叠区域。
- 补充必要单测和手册说明。

### 范围外
- 不恢复独立 Prompt Ops 面板。
- 不实现 rollback/preview 全套交互（本轮仅 `apply` 闭环）。
- 不新增本地磁盘持久化。

## 接口 / 数据
- 前端状态扩展（`AgentChatDraftState`）：
  - `profile_loaded_agent_id: Option<String>`
  - `profile_system_prompt: String`
  - `profile_short_term_goal: String`
  - `profile_long_term_goal: String`
- 数据来源：
  - `ViewerState.events` 中 `WorldEventKind::AgentPromptUpdated` 的最新 profile。
  - 无事件时回退 `AgentPromptProfile::for_agent(agent_id)`。
- 协议复用：
  - `ViewerRequest::PromptControl { command: PromptControlCommand::Apply { request } }`

## 里程碑
- M1：设计文档与项目管理文档完成。
- M2：Chat Panel 折叠区集成 Agent Prompt 字段编辑。
- M3：`apply` 链路接入与状态提示完成。
- M4：测试、文档、devlog 收口完成。

## 风险
- 风险：仅 `apply` 不含 preview/rollback，误操作可恢复路径较弱。
  - 缓解：保留“加载当前配置”快捷回填，后续可增量补 rollback。
- 风险：当前若未消费 `PromptControlAck/Error`，用户难以感知服务端拒绝原因。
  - 缓解：先提供本地“已发送请求”状态，并在后续迭代加入回执态展示。
- 风险：折叠区内容增多导致窄屏拥挤。
  - 缓解：子区分组 + 默认折叠，输入框行数保持紧凑。
