# Viewer 发行体验改造（项目管理文档）

## 任务拆解
- [x] VGRO0：建立设计文档与项目管理文档。
- [x] VGRO1：实现体验模式资源与环境变量解析（Player/Director）。
- [x] VGRO2：实现按模式覆盖默认布局与默认模块可见性。
- [x] VGRO3：实现 Player 模式入口可发现性增强（提示文案/入口行为）。
- [ ] VGRO4：补充/更新测试并执行 viewer 相关回归。
- [ ] VGRO5：回顾设计文档与项目文档，更新状态并收口。

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/right_panel_module_visibility.rs`
- `crates/agent_world_viewer/src/ui_state_types.rs`
- `crates/agent_world_viewer/src/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VGRO3 完成，VGRO4 进行中。
- 阻塞项：无。
- 最近更新：已完成 Player 模式入口提示卡片与 Tab 快捷键开关，进入回归验证阶段。
