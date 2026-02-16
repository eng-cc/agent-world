# Viewer Agent Chat 左右双 Panel 与会话历史（项目管理文档）

## 任务拆解
- [x] VCD1 输出设计文档（`doc/world-simulator/viewer-chat-dual-panel.md`）
- [x] VCD2 输出项目管理文档（本文件）
- [ ] VCD3 新增左侧 Chat History Panel（会话列表）
- [ ] VCD4 右侧 Chat 区升级为会话视图（消息气泡 + 输入发送联动）
- [ ] VCD5 聊天历史聚合模型实现（trace -> 会话列表/会话详情）
- [ ] VCD6 更新 3D 输入命中边界，避让左右 Panel
- [ ] VCD7 补充/更新测试并执行回归（`test_tier_required` 最小闭环 + wasm check）
- [ ] VCD8 回写文档状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/ui_state_types.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/right_panel_module_visibility.rs`

## 状态
- 当前阶段：VCD1-VCD2 已完成，进入实现阶段（VCD3-VCD6）。
- 下一步：先完成左右 panel 与会话聚合，再执行测试与文档收口。
- 最近更新：VCD2 完成（2026-02-16）。
