# Viewer Chat 预设 Prompt 编辑区设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-prompt-presets.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-prompt-presets.project.md`

## 1. 设计定位
定义 Chat Panel 内部的预设 Prompt 折叠编辑区，以一个入口承载聊天预设的新增、编辑、删除和一键填充，并收敛掉独立 Prompt Ops UI。

## 2. 设计结构
- UI 收敛层：移除 Prompt Ops 模式入口与面板接线。
- 预设编辑层：在 Chat Panel 中提供折叠的预设列表与编辑区。
- 填充复用层：选中预设后将内容写入聊天输入框，继续走既有 `AgentChat` 发送链路。
- 运行态层：预设先保留为本地运行态，不做跨会话持久化。

## 3. 关键接口 / 入口
- `Prompt Presets` 折叠区
- `AgentChatDraftState.prompt_presets`
- `selected_preset_index`
- `input_message`
- `ViewerRequest::AgentChat`

## 4. 约束与边界
- 不改后端 `prompt_control.*` 协议定义。
- 不新增磁盘持久化与消息流渲染逻辑改造。
- Prompt Ops UI 下线后底层协议仍保留，以便后续恢复独立运营面板。
- 窄屏下预设编辑区默认折叠，控制空间占用。

## 5. 设计演进计划
- 先移除 Prompt Ops UI 入口。
- 再补 Chat Panel 预设编辑与填充能力。
- 最后通过测试和手册把 Chat 预设收敛为唯一主入口。
