# Viewer 发行体验改造（第十阶段：项目管理文档）

审计轮次: 2

## 审计备注
- 主项目入口文档：`doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.project.md`。
- 本文件仅维护增量任务。

## 任务拆解（含 PRD-ID 映射）
- [x] VRI10P0：建立第十阶段设计文档与项目管理文档。
- [x] VRI10P1：修复教程进度跳步并强化第 4 步动作文案（含单测）。
- [x] VRI10P2：实现隐藏态引导层减噪策略并完成定向验证。
- [x] VRI10P3：执行回归与 Playwright 新手流程复测并完成文档收口。

## 依赖
- doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase10.prd.md
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_entry.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide_progress_tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_mission_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VRI10P3 完成（第十阶段收口）。
- 阻塞项：无。
- 最近更新：S5 回归（`agent_world_viewer` 全量测试 + wasm check）与 S6 Playwright 新手流程复测完成，阶段任务全部闭环。
