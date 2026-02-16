# Viewer Chat 独立最右侧 Panel（项目管理文档）

## 任务拆解
- [x] DCR1 输出设计文档（`doc/world-simulator/viewer-chat-dedicated-right-panel.md`）
- [x] DCR2 输出项目管理文档（本文件）
- [ ] DCR3 改造 EGUI 布局：拆分独立 Chat 右侧 Panel（最右侧）
- [ ] DCR4 适配右侧总占用宽度并补充/更新测试
- [ ] DCR5 更新手册、回写状态与 devlog 收口

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/ui_state_types.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：DCR1-DCR2 已完成，进入 DCR3。
- 下一步：完成双右侧面板布局改造并联动边界判定。
- 最近更新：2026-02-16。
