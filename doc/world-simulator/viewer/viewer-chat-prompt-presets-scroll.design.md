# Viewer Chat 预设展开区滚动设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-prompt-presets-scroll.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-prompt-presets-scroll.project.md`

## 1. 设计定位
定义 Chat Panel 中预设 Prompt 展开区的内部滚动容器，使在小窗口或低分辨率下，预设编辑和 Agent Prompt 字段仍能被完整访问。

## 2. 设计结构
- 高度预算层：为预设展开区计算可用高度上限。
- ScrollArea 容器层：把预设编辑和 Agent Prompt 草稿区包裹进垂直滚动区。
- 焦点兼容层：保持输入焦点、Enter 发送和滚动行为互不干扰。
- 手册同步层：补充滚动交互说明和小窗口使用口径。

## 3. 关键接口 / 入口
- `ScrollArea`
- 预设展开区最大滚动高度常量
- `egui_right_panel_chat.rs`
- Chat 输入焦点状态

## 4. 约束与边界
- 不改 Prompt 语义、apply 逻辑与消息发送逻辑。
- 过小高度下布局要稳定，避免抖动。
- 滚动容器不能破坏 IME 和 Enter 发送路径。
- 本轮只做内部滚动，不扩展更多布局模式。

## 5. 设计演进计划
- 先建立展开区高度计算。
- 再接入滚动容器与焦点兼容处理。
- 最后通过布局测试和手册收口小窗口可用性。
