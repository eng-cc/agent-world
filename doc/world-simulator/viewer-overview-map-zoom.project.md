# Viewer 全览图缩放切换（项目管理文档）

## 任务拆解
- [x] OVZ1.1 输出设计文档（`doc/world-simulator/viewer-overview-map-zoom.md`）
- [x] OVZ1.2 输出项目管理文档（本文件）
- [ ] OVZ2.1 引入 `TwoDZoomTier` 与缩放阈值状态机（含迟滞）
- [ ] OVZ2.2 调整 2D 默认缩放口径，默认进入细节态（适合观察 Agent）
- [ ] OVZ3.1 全览图可见性联动（`TwoDMapMarker` 与细节实体）
- [ ] OVZ3.2 更新/新增相关单测并执行 `test_tier_required` 回归
- [ ] OVZ4.1 更新总项目文档状态与开发日志

## 依赖
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/location_fragment_render.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：OVZ1 已完成，进入 OVZ2。
- 下一步：实现缩放层级状态机与默认倍率口径调整。
- 最近更新：2026-02-15（初始化任务与拆解）。
