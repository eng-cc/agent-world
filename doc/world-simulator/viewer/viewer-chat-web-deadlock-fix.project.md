# Viewer Chat Web 锁重入修复（项目管理文档）

## 任务拆解
- [x] WDF1 完成 Web Chat 卡死修复、回车发送修复、闭环验证与文档收口

## 依赖
- `Cargo.toml`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/wasm_egui_input_bridge.rs`
- `doc/devlog/2026-02-16.md`

## 状态
- 当前阶段：WDF1 已完成。
- 闭环结果：Web 模式可稳定运行并可在 Chat 面板输入消息发送给 Agent（含“消息已发送”状态回显与 WS payload 证据）。
- 最近更新：2026-02-16。
