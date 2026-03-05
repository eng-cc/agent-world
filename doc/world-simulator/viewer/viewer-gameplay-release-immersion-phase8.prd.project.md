# Viewer 发行体验改造（第八阶段：项目管理文档）

审计轮次: 2

## 审计备注（2026-03-05 ROUND-002 物理合并）
- 本阶段任务已合并入 `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.project.md`。
- 当前替代入口：
  - `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.md`
  - `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.prd.project.md`
- 本文件保留为阶段追溯，不再作为当前执行基线。

## 任务拆解（含 PRD-ID 映射）
- [x] VRI8P0：建立第八阶段设计文档与项目管理文档。
- [x] VRI8P1：收敛“下一步目标卡”与“新手引导卡”共存策略，去除重复提示。
- [x] VRI8P2：改造任务 HUD 的自适应锚点与展开态紧凑模式。
- [x] VRI8P3：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase8.prd.md
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：phase8 已完成（VRI8P0~VRI8P3 全部完成）。
- 阻塞项：无。
- 最近更新：完成 phase8 回归与 Web 闭环验收，确认提示去重与任务 HUD 收敛策略在真实 Web 运行中生效。
