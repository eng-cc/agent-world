# Viewer Chat 独立最右侧 Panel 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-dedicated-right-panel.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-dedicated-right-panel.project.md`

## 1. 设计定位
定义 Chat 从综合右侧面板拆分为独立最右侧 Panel 的布局方案，使聊天能力与观察/控制模块解耦，同时保持现有 AgentChat 协议与状态语义不变。

## 2. 设计结构
- 独立面板层：新增最右侧 Chat SidePanel，承载聊天内容与输入。
- 综合面板收敛层：原右侧综合面板移除聊天渲染。
- 宽度聚合层：右侧总占用宽度更新为“综合面板 + Chat Panel”之和。
- 显隐兼容层：继续复用 `show_chat` 控制 Chat Panel 显隐。

## 3. 关键接口 / 入口
- `RightPanelModuleVisibilityState.show_chat`
- `AgentChatDraftState`
- `ChatInputFocusSignal`
- `RightPanelWidthState.width_px`
- `egui_right_panel.rs` / `egui_right_panel_chat.rs`

## 4. 约束与边界
- 不改 `AgentChat` 协议、消息聚合算法与气泡样式。
- Prompt Ops 模式继续保持不展示 Chat，避免新增交互分支。
- 双面板场景下 3D 输入命中边界必须以总宽度为准。
- 窄屏下要允许拖拽调整宽度，避免严重挤压视口。

## 5. 设计演进计划
- 先拆出独立 Chat Panel 和显隐逻辑。
- 再同步右侧总宽度与输入边界判定。
- 最后补测试与手册，固定双面板行为基线。
