# Viewer 发行体验改造（第五阶段：项目管理文档）

- 对应设计文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase5.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase5.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] VRI5P0：建立第五阶段设计文档与项目管理文档。
- [x] VRI5P1：实现 Player 右侧面板沉浸式结构重构（边缘呼出入口 + 面板宽度预算约束）。
- [x] VRI5P2：实现新手任务闭环提示增强，并拆分 player_experience 模块（Rust 文件长度合规）。
- [x] VRI5P3：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase5.prd.md
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_layout.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_entry.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VRI5P0~VRI5P3 全部完成（第五阶段收口）。
- 阻塞项：无。
- 最近更新：完成 S6 Web 闭环验收、回归与文档收口。
