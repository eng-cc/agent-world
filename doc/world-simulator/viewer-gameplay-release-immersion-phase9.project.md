# Viewer 发行体验改造（第九阶段：项目管理文档）

## 任务拆解
- [x] VRI9P0：建立第九阶段设计文档与项目管理文档。
- [ ] VRI9P1：实现顶部布局预设条减噪策略（隐藏态移除 + 锚点重排）并补单测。
- [ ] VRI9P2：实现隐藏态任务 HUD“一键指挥 Agent”入口并补单测。
- [ ] VRI9P3：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_mission_tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_layout_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VRI9P0 完成，VRI9P1 进行中。
- 阻塞项：无。
- 最近更新：第九阶段建档完成，进入“顶部减噪 + 指挥链路前置”实现。
