# [已归档] Viewer Agent Chat 左右双 Panel 与会话历史（项目管理文档）

> 归档标记（2026-02-17）：该任务对应的设计已被后续方案替代。
> 过时原因：项目项包含“新增左侧 Chat History Panel（VCD3）”，与当前实现不一致（当前无左侧 Chat History 面板）。
> 替代文档：`doc/world-simulator/viewer-chat-dedicated-right-panel.project.md`、`doc/world-simulator/viewer-chat-right-panel-polish.project.md`。

## 任务拆解
- [x] VCD1 输出设计文档（`doc/world-simulator/viewer-chat-dual-panel.md`）
- [x] VCD2 输出项目管理文档（本文件）
- [x] VCD3 新增左侧 Chat History Panel（会话列表）
- [x] VCD4 右侧 Chat 区升级为会话视图（消息气泡 + 输入发送联动）
- [x] VCD5 聊天历史聚合模型实现（trace -> 会话列表/会话详情）
- [x] VCD6 更新 3D 输入命中边界，避让左右 Panel
- [x] VCD7 补充/更新测试并执行回归（`test_tier_required` 最小闭环 + wasm check）
- [x] VCD8 回写文档状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/ui_state_types.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/right_panel_module_visibility.rs`

## 状态
- 当前阶段：已完成（VCD1-VCD8）。
- 下一步：等待验收与后续迭代需求。
- 最近更新：VCD8 完成（2026-02-16）。
