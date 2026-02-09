# Viewer 右侧 2D UI 迁移到 bevy_egui（项目管理文档）

## 任务拆解
- [x] ER1：输出设计文档（`doc/world-simulator/viewer-egui-right-panel.md`）
- [x] ER2：输出项目管理文档（本文件）
- [x] ER3：实现 EGUI 右侧 SidePanel 骨架并接入调度
- [x] ER4：迁移顶部控制、状态摘要、详情、事件、诊断、联动、时间轴、覆盖层到 EGUI
- [x] ER5：3D 视口/鼠标命中边界改为读取 EGUI 面板宽度
- [x] ER6：移除旧 Bevy UI 右侧面板启动与交互调度（清理遗留代码并收敛告警）
- [x] ER7：补充/更新测试并完成截图闭环验证
- [x] ER8：更新任务日志并完成阶段提交

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/timeline_controls.rs`
- `crates/agent_world_viewer/src/event_click_list.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/copyable_text.rs`
- `crates/agent_world_viewer/src/world_overlay.rs`
- `crates/agent_world_viewer/src/button_feedback.rs`
- `crates/agent_world_viewer/src/panel_layout.rs`
- `crates/agent_world_viewer/src/panel_scroll.rs`

## 状态
- 当前阶段：已完成 ER1~ER8（迁移主线完成）
- 下一阶段：按需继续做细节交互优化（非迁移阻塞项）
- 最近更新：完成旧 Bevy 右侧 UI 遗留清理，`agent_world_viewer` 编译告警归零并完成截图回归（2026-02-09）
