# Viewer Chat 右侧收敛布局设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-right-panel-polish.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-right-panel-polish.project.md`

## 1. 设计定位
定义 Chat 收敛到右侧单区块的最终布局：聊天记录在上、Agent 选择与输入在下，并去除左侧历史栏，使 Chat 交互路径更稳定、Web 闭环更容易验证。

## 2. 设计结构
- 历史栏清理层：移除左侧 Chat History SidePanel。
- 右侧重排层：右侧 Chat 区按“记录在上、输入在下”重新布局。
- 边界同步层：3D 输入命中边界仅避让右侧 panel。
- 闭环验证层：通过 Web Playwright 截图验证布局与交互可用性。

## 3. 关键接口 / 入口
- `ViewerState.decision_traces[].llm_chat_messages[]`
- `AgentChatDraftState`
- `egui_right_panel_chat.rs`
- `camera_controls.rs`
- `run-viewer-web.sh`

## 4. 约束与边界
- 不改 `AgentChat` 协议结构和会话持久化能力。
- 右侧布局重排不能破坏 IME 聚焦与 Enter 发送。
- 左侧历史栏移除后，相机/拾取边界要同步清理旧依赖。
- 本轮只收敛布局，不扩展额外业务功能。

## 5. 设计演进计划
- 先移除左侧历史栏并收敛到右侧单区块。
- 再重排右侧内容层级与输入区域。
- 最后通过 3D 边界回归和 Web 截图取证完成验收。
