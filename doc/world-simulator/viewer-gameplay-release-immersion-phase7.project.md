# Viewer 发行体验改造（第七阶段：项目管理文档）

## 任务拆解
- [x] VRI7P0：建立第七阶段设计文档与项目管理文档。
- [x] VRI7P1：实现 Player 布局预设（任务/指挥/情报）与快捷切换条。
- [x] VRI7P2：实现隐藏状态“直接指挥”入口，并接入指挥预设联动。
- [x] VRI7P3：调整 Player 默认模块可见性与面板宽度预算（世界优先）。
- [ ] VRI7P4：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_entry.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VRI7P3 完成，VRI7P4 进行中。
- 阻塞项：无。
- 最近更新：Player 默认模块改为“可直接指挥”，并新增 Player 专用宽度预算，右侧主/聊天面板在宽屏下不再挤占过多世界视野。
