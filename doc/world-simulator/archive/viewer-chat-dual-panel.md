# [已归档] Agent World Simulator：Viewer Agent Chat 左右双 Panel 与会话历史（设计文档）

> 归档标记（2026-02-17）：该设计已过时，不再作为当前实现依据。
> 过时原因：文档要求“左侧 Chat History SidePanel + 右侧 Chat Panel”，但当前实现已移除左侧历史栏，仅保留右侧主信息面板与最右侧 Chat 面板（见 `crates/agent_world_viewer/src/egui_right_panel.rs`）。
> 替代文档：`doc/world-simulator/viewer-chat-dedicated-right-panel.md`、`doc/world-simulator/viewer-chat-right-panel-polish.md`。

## 目标
- 将 Viewer 中 Agent Chat 从“右侧单模块”升级为“左右双 Panel”形态：
  - 左侧新增 Chat History Panel（会话列表）。
  - 右侧保留/升级 Chat Panel（会话内容 + 输入发送）。
- 增强聊天记录可读性，支持按会话查看历史消息，交互形态参考 ChatGPT。
- 保持现有 Agent Chat 协议、LLM trace 数据链路与 Web/Native 运行方式兼容。

## 范围

### 范围内
- 新增左侧 EGUI `SidePanel`，用于展示会话历史列表（含标题、最近更新时间、预览）。
- 右侧 Chat 区升级为会话视图：
  - 展示当前会话消息流（玩家/Agent/工具/系统）。
  - 输入框与发送按钮保留，继续通过 `ViewerRequest::AgentChat` 发送。
  - 消息使用接近 ChatGPT 的“气泡式”展示，提升可读性。
- 增加聊天历史聚合逻辑：从 `decision_traces.llm_chat_messages` 生成会话列表与会话消息。
- 更新 3D 交互命中边界：避让左右 Panel，避免面板区域触发 3D 相机/拾取。
- 补充回归测试：
  - `test_tier_required`：会话聚合、边界判定、模块可见性/核心 UI 逻辑。
  - `test_tier_full`：沿用现有 Web 闭环口径做人工/脚本回归。

### 范围外
- 不改动 viewer 协议字段定义（沿用现有 `AgentChatRequest`/`DecisionTrace`）。
- 不实现服务端长期会话存储（仅依赖现有 trace 数据在前端聚合展示）。
- 不改动 Prompt Ops 面板形态与业务流程。

## 接口 / 数据

### 1) UI 状态
- 新增左侧聊天历史面板宽度状态资源（例如 `ChatHistoryPanelWidthState`），用于输入命中避让。
- 复用现有聊天草稿状态，并补充“当前会话选择”字段（会话 ID）。

### 2) 会话聚合模型（Viewer 端）
- 输入：`ViewerState.decision_traces[].llm_chat_messages[]`。
- 聚合规则：按 `agent_id` + 会话起点拆分消息窗口（玩家发言作为会话分隔优先信号），生成：
  - 会话 ID
  - 标题（优先取首条玩家消息摘要）
  - 最近更新时间
  - 消息列表
- 输出：
  - 左侧用于历史列表。
  - 右侧用于当前会话正文渲染。

### 3) 3D 输入边界
- 现状仅避让右侧宽度。
- 改造后统一使用“左宽 + 右宽”进行 3D 区域判定：
  - 仅在中间视口区域响应相机拖拽/缩放与拾取。

## 里程碑
- M1：设计文档与项目管理文档完成。
- M2：会话聚合与左右双 Panel 骨架完成。
- M3：聊天记录展示升级（气泡、会话选择、发送联动）完成。
- M4：测试回归、文档回写、devlog 收口完成。

## 风险
- 会话切分准确性风险：仅靠 trace 推断会话边界，极端情况下与用户认知不一致。
  - 缓解：优先基于玩家发言切分；无玩家发言时保底为单会话。
- 屏幕空间风险：左右 panel 同时出现可能压缩 3D 视口。
  - 缓解：限制左右 panel 宽度范围，并默认仅在 chat 模块开启时显示左侧历史。
- 数据量风险：trace 累积导致渲染压力上升。
  - 缓解：限制会话与消息窗口上限，只展示最近 N 条。
