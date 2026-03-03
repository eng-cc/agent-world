# Viewer 双视角（2D/3D）切换（项目管理文档）

## 任务拆解
- [x] DV1：输出设计文档（`viewer-dual-view-2d-3d.md`）
- [x] DV2：输出项目管理文档（本文件）
- [x] DV3：实现 `ViewerCameraMode` 资源与默认 2D 视角
- [x] DV4：实现 2D/3D 相机模式同步逻辑
- [x] DV5：在右侧顶部增加 2D/3D 切换按钮与 i18n 文案
- [x] DV6：调整 Agent 默认颜色为更鲜艳配色
- [x] DV7：补充/更新单元测试与 UI 测试
- [x] DV8：执行 fmt/check/test 与截图回看（如需要）
- [x] DV9：更新项目文档状态与开发日志
- [x] DV10：2D 视角隐藏 world floor/bounds 背景面，避免蓝黑底色干扰
- [x] DV11：3D 视角同步隐藏 world floor/bounds（两种视角均无蓝黑底色）

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/i18n.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `doc/devlog/2026-02-09.md`

## 状态
- 当前阶段：DV1~DV11 已完成。
- 下一步：按需补充 2D/3D 切换的视觉回归 snapshot（例如 triad 场景双视角基线图）。
- 最近更新：2026-02-09。
