# Viewer 发行体验改造（第九阶段：项目管理文档）

审计轮次: 2

## 审计备注（2026-03-05 ROUND-002 物理合并）
- 本阶段任务已合并入 `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.project.md`。
- 当前替代入口：
  - `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.md`
  - `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.project.md`
- 本文件保留为阶段追溯，不再作为当前执行基线。

## 任务拆解（含 PRD-ID 映射）
- [x] VRI9P0：建立第九阶段设计文档与项目管理文档。
- [x] VRI9P1：实现顶部布局预设条减噪策略（隐藏态移除 + 锚点重排）并补单测。
- [x] VRI9P2：实现隐藏态任务 HUD“一键指挥 Agent”入口并补单测。
- [x] VRI9P3：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase9.prd.md
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_mission_tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_layout_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：phase9 已完成（VRI9P0~VRI9P3 全部完成）。
- 阻塞项：无。
- 最近更新：完成 phase9 回归与 Web 闭环验收，确认“顶部减噪 + 隐藏态一键指挥”在真实 Web 运行中生效。
