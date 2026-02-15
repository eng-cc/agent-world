# Viewer 2D/3D WASD 相机移动（项目管理文档）

## 任务拆解

### WCM1 文档与对齐
- [x] WCM1.1 输出设计文档（`doc/world-simulator/viewer-wasd-camera-navigation.md`）
- [x] WCM1.2 输出项目管理文档（本文件）
- [x] WCM1.3 在总项目文档挂载任务入口

### WCM2 输入系统实现
- [x] WCM2.1 `camera_controls` 接入 WASD 输入轴（W/A/S/D）
- [x] WCM2.2 2D/3D 统一接入键盘平移（仅移动 `OrbitCamera.focus`）
- [x] WCM2.3 输入冲突保护（EGUI 键盘占用时禁用）

### WCM3 测试与回归
- [x] WCM3.1 单测：WASD 输入轴映射
- [x] WCM3.2 单测：2D/3D 模式下 WASD 位移行为
- [x] WCM3.3 执行 `test_tier_required` 最小回归（camera_controls + `cargo check`）

### WCM4 收口
- [x] WCM4.1 更新 `doc/viewer-manual.md` 交互说明
- [x] WCM4.2 更新项目文档状态与开发日志

## 依赖
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `doc/viewer-manual.md`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：WCM1~WCM4 全部完成。
- 下一步：无（本任务收口完成）。
- 最近更新：2026-02-15（完成手册/日志收口并更新总项目状态）。
