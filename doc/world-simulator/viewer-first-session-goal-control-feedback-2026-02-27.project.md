# Viewer 首局目标与控制语义可解释反馈优化（2026-02-27）项目管理文档

## 任务拆解
- [x] T0 建立设计文档与项目文档，明确边界与接口
- [ ] T1 首局目标改造：输出 1 个主目标 + 2 个短目标，并接入引导 HUD
- [ ] T2 控制语义可发现：补充 `describeControls` 与示例填充入口
- [ ] T3 输入可解释反馈：`sendControl` 结构化返回 + `getState.lastControlFeedback`
- [ ] T4 测试与收口：补充/更新测试、回写文档状态、沉淀任务日志

## 依赖
- `doc/game-test.md`
- `doc/playability_test_result/card_2026_02_27_15_05_28.md`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`

## 状态
- 当前阶段：进行中（T1）
- 最近更新：2026-02-27
