# Agent World Simulator：Viewer Chat 预设 Prompt 编辑区（设计文档）

## 目标
- 删除现有 `Prompt Ops` 运营模块入口，避免与聊天链路并行维护两套 Prompt 交互面板。
- 在最右侧 Chat Panel 内新增一个“可展开的小区域”，用于编辑和维护预设 prompt。
- 让玩家可快速把预设 prompt 填充到聊天输入框，再走既有 `AgentChat` 发送链路。

## 范围

### 范围内
- UI 结构调整：
  - 移除右侧主面板中的 `Observe/Prompt Ops` 模式切换与 Prompt Ops 面板渲染。
  - 保留右侧独立 Chat Panel，并在其中加入折叠区 `Prompt Presets`（默认折叠）。
- 预设 prompt 编辑能力（Viewer 本地态）：
  - 预设项列表展示。
  - 选择后可编辑“名称/内容”。
  - 支持新增和删除预设项。
  - 支持“一键填充到聊天输入框”。
- 兼容现有发送链路：
  - 继续通过 `ViewerRequest::AgentChat` 发送，服务端协议不变。
- 代码清理：
  - 删除 Prompt Ops UI 模块接线与不再使用的分支/测试。

### 范围外
- 不改动 `agent_world` live server 的 `prompt_control.*` 协议定义。
- 不做预设 prompt 的磁盘持久化（本轮仅保留运行期本地状态）。
- 不改动会话聚合算法、消息流渲染样式和 AgentChat 协议字段。

## 接口 / 数据
- `AgentChatDraftState` 新增预设编辑状态：
  - `preset_panel_open: bool`
  - `prompt_presets: Vec<PromptPresetDraft>`
  - `selected_preset_index: usize`
- 新增 `PromptPresetDraft`：
  - `name: String`
  - `content: String`
- 交互行为：
  - 点击“填充到输入框”后，将选中预设内容写入 `input_message`。
  - 输入框发送逻辑保持不变（`Enter` 发送，`Shift+Enter` 换行）。

## 里程碑
- M1：设计文档与项目管理文档完成。
- M2：移除 Prompt Ops UI 入口与模块接线。
- M3：Chat Panel 预设 prompt 折叠编辑区完成。
- M4：测试回归、文档回写、devlog 与提交收口完成。

## 风险
- 风险：移除 Prompt Ops 后，历史“线上改写 Agent profile”入口下线，可能影响既有运营操作路径。
  - 缓解：明确本轮需求定位为“聊天侧预设编辑”，并保留底层协议以便后续按需恢复独立运营面板。
- 风险：预设编辑区加入后，Chat panel 在窄屏下可用空间变小。
  - 缓解：默认折叠，且编辑区布局采用紧凑控件。
- 风险：无持久化可能导致刷新后预设丢失。
  - 缓解：在文档中明确本轮为运行期状态，后续可追加持久化任务。
