# Viewer 2D/3D 可视化清晰度修复（项目管理文档）

## 任务拆解
- [x] CFX1.1 输出设计文档（`doc/world-simulator/viewer-2d-3d-clarity-fix.md`）
- [x] CFX1.2 输出项目管理文档（本文件）
- [ ] CFX2.1 修复 Location 渲染尺度单位映射（`radius_cm -> world units`）
- [ ] CFX2.2 补充/更新 Location 尺度相关单测
- [ ] CFX3.1 修复 2D 自动聚焦后正交缩放同步
- [ ] CFX3.2 补充自动聚焦 2D 缩放回归测试
- [ ] CFX4.1 收敛右侧模块默认可见性（降低首次信息密度）
- [ ] CFX4.2 执行 test_tier_required 回归与截图闭环
- [ ] CFX4.3 更新项目文档状态与开发日志

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/auto_focus.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/right_panel_module_visibility.rs`
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：CFX1 已完成，CFX2~CFX4 待执行。
- 下一步：执行 CFX2（Location 尺度修复）并补齐测试。
- 最近更新：2026-02-15。
