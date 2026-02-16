# Agent World Simulator：Viewer Chat Web IME EGUI 输入桥接（项目管理文档）

## 任务拆解
- [x] CIB1 输出设计文档（`doc/world-simulator/viewer-chat-ime-egui-bridge.md`）
- [x] CIB2 输出项目管理文档（本文件）
- [x] CIB3 实现 wasm EGUI IME bridge（DOM 事件桥接 + EGUI 事件注入）
- [x] CIB4 接入启动流程并完成本地编译/测试回归
- [x] CIB5 Web 闭环验证（Playwright 快照、console、截图）
- [x] CIB6 文档回写、devlog、提交收口
- [x] CIB7 现场反馈回修：桥接焦点判定改为 `wants_keyboard_input`，并使用 IME Commit 事件
- [x] CIB8 回归验证与提交收口（CIB7）
- [x] CIB9 现场反馈回修：修复桥接焦点抖动导致输入框无法聚焦
- [x] CIB10 回归验证与提交收口（CIB9）
- [x] CIB11 现场反馈回修：为聊天 `TextEdit` 增加显式焦点信号并驱动 wasm IME bridge 聚焦
- [x] CIB12 回归验证与提交收口（CIB11）
- [x] CIB13 现场反馈回修：确认 `winit web IME` 框架限制并将 bridge 复用到 `bevy_egui` 的 `egui_text_agent`
- [x] CIB14 回归验证与提交收口（CIB13）
- [x] CIB15 现场反馈回修：修复聊天输入框二次点击导致 bridge 丢失 DOM 焦点
- [ ] CIB16 回归验证与提交收口（CIB15）

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/wasm_egui_input_bridge.rs`（新增）
- `crates/agent_world_viewer/Cargo.toml`

## 状态
- 当前阶段：进行中（CIB16）
- 下一步：你侧手工复测“首次点击 + 二次点击（淡蓝高亮）+ 再输入中文”链路
- 最近更新：CIB15 完成（2026-02-16）
