# Viewer Chat 中文输入兼容修复设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-ime-cn-input.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-ime-cn-input.project.md`

## 1. 设计定位
定义 Web Viewer 中 Chat 输入框对中文输入法的兼容修复方案，通过调整 Web 输入事件处理配置，恢复浏览器端中文输入上屏能力。

## 2. 设计结构
- Web 输入配置层：调整 Viewer 窗口初始化和输入事件配置以适配中文 IME。
- Chat 输入兼容层：确保候选词提交与文本上屏行为稳定。
- 闭环验证层：通过 wasm check、Web 运行和截图验证中文输入路径。
- 兼容保护层：英文输入与消息发送路径保持不变。

## 3. 关键接口 / 入口
- `app_bootstrap.rs`
- `egui_right_panel_chat.rs`
- Web 输入事件配置
- `run-viewer-web.sh`

## 4. 约束与边界
- 不新增协议字段，不改 `ViewerRequest::AgentChat` 结构。
- 修复聚焦于浏览器输入兼容，不扩展 native 平台行为。
- 浏览器差异带来的边缘行为要有明确回归口径。
- 本轮不修改上游 `bevy/bevy_egui` 源码。

## 5. 设计演进计划
- 先固定 Web 输入配置修复点。
- 再验证中文候选词上屏与英文路径不回归。
- 最后用 Web 闭环与文档回写完成收口。
