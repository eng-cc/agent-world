# Viewer 右侧 2D UI 迁移到 bevy_egui（项目管理文档）

## 任务拆解
- [x] ER1：输出设计文档（`doc/world-simulator/viewer-egui-right-panel.md`）
- [x] ER2：输出项目管理文档（本文件）
- [ ] ER3：实现 EGUI 右侧 SidePanel 骨架并接入调度
- [ ] ER4：迁移顶部控制、状态摘要、详情、事件、诊断、联动、时间轴、覆盖层到 EGUI
- [ ] ER5：3D 视口/鼠标命中边界改为读取 EGUI 面板宽度
- [ ] ER6：移除旧 Bevy UI 右侧面板启动与交互调度
- [ ] ER7：补充/更新测试并完成截图闭环验证
- [ ] ER8：更新任务日志并提交

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/timeline_controls.rs`
- `crates/agent_world_viewer/src/event_click_list.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`

## 状态
- 当前阶段：ER3（准备接入 SidePanel 骨架）
- 下一阶段：迁移交互控制与详情内容
- 最近更新：完成迁移设计与任务拆解（2026-02-09）
