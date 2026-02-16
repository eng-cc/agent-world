# Viewer 控制区精简与高级调试折叠（项目管理文档）

## 任务拆解
- [x] ADF1 输出设计文档（`doc/world-simulator/viewer-control-advanced-debug-folding.md`）
- [x] ADF2 输出项目管理文档（本文件）
- [ ] ADF3 实现 EGUI 控制区改造（播放/暂停单按钮 + 高级调试折叠）
- [ ] ADF4 补充/更新测试并执行回归（`test_tier_required` 最小闭环）
- [ ] ADF5 更新手册、回写状态与 devlog 收口

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/i18n.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：ADF1-ADF2 已完成，进入 ADF3。
- 下一步：完成控制区改造并补齐测试。
- 最近更新：2026-02-16。
