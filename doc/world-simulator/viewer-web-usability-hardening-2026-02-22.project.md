# Viewer Web 可操作性与舒适度加固（项目管理文档）

## 任务拆解
- [x] VWU0：建立设计文档与项目管理文档。
- [x] VWU1：实现窄屏布局加固（保证最小主画布宽度，聊天降级到主面板内）。
- [x] VWU2：修复 websocket `onerror` 事件类型不匹配导致的运行时异常。
- [x] VWU3：实现连接异常自动重连（退避）与友好错误文案。
- [x] VWU4：补充/更新测试并执行 required 相关回归。
- [x] VWU5：回顾设计文档与项目文档，更新状态并收口。

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/tests.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VWU0~VWU5 全部完成（已收口）。
- 阻塞项：无。
- 最近更新：2026-02-22 22:00 CST。
