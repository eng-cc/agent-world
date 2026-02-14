# Viewer 2D 可视化精修（项目管理文档）

## 任务拆解

### V2D0 文档与挂载
- [x] V2D0.1 输出设计文档（`doc/world-simulator/viewer-2d-visual-polish.md`）
- [x] V2D0.2 输出项目管理文档（本文件）
- [x] V2D0.3 在总项目文档挂载分册入口

### V2D1 地图符号层（2D）
- [x] V2D1.1 Location 2D 地图符号（平面底板/中心点）
- [x] V2D1.2 Agent 2D 地图符号（平面高亮标记）
- [x] V2D1.3 2D/3D 模式切换联动（2D 显示、3D 隐藏）

### V2D2 标签可读性增强（2D）
- [ ] V2D2.1 标签 LOD 增加 2D 配置分支（距离/容量/遮挡）
- [ ] V2D2.2 单测补齐（2D 配置与可见性行为）
- [ ] V2D2.3 回归验证与截图闭环

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/scene_helpers_entities.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/label_lod.rs`
- `crates/agent_world_viewer/src/tests_scene_entities.rs`
- `doc/world-simulator/viewer-dual-view-2d-3d.md`

## 状态
- 当前阶段：V2D1 已完成，进入 V2D2。
- 下一阶段：完成标签 LOD 增强、回归测试和截图闭环。
- 最近更新：完成 V2D1（2D 地图符号层 + 模式联动，2026-02-14）。
