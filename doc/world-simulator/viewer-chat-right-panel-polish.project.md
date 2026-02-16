# Viewer Chat 右侧收敛布局与闭环验收（项目管理文档）

## 任务拆解
- [x] CRP1 输出设计文档（`doc/world-simulator/viewer-chat-right-panel-polish.md`）
- [x] CRP2 输出项目管理文档（本文件）
- [ ] CRP3 移除左侧 Chat History Panel，收敛到右侧
- [ ] CRP4 重排右侧 Chat：聊天记录在上，Agent 选择与输入发送在下
- [ ] CRP5 回归 3D 输入边界（仅避让右侧 panel）
- [ ] CRP6 执行回归测试并完成 Web 闭环截图取证
- [ ] CRP7 回写文档状态与 devlog，提交收口

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/ui_state_types.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `scripts/run-viewer-web.sh`

## 状态
- 当前阶段：CRP1-CRP2 已完成，进入实现阶段（CRP3-CRP5）。
- 下一步：完成布局调整后执行 Web 闭环截图并收口。
- 最近更新：CRP2 完成（2026-02-16）。
