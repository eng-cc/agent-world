# Viewer 发行体验改造（第六阶段：项目管理文档）

## 任务拆解
- [x] VRI6P0：建立第六阶段设计文档与项目管理文档。
- [x] VRI6P1：实现 Player 电影化开场层（连接后短时叙事覆盖 + 自动淡出）。
- [ ] VRI6P2：实现任务驱动 HUD（主任务/步骤进度/当前动作提示）。
- [ ] VRI6P3：实现任务推进奖励反馈强化（完成态文本与视觉层级）。
- [ ] VRI6P4：实现 Player 小地图卡片（位置缩略 + 选中高亮联动）。
- [ ] VRI6P5：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VRI6P1 完成，VRI6P2 进行中。
- 阻塞项：无。
- 最近更新：Player 电影化开场层已落地并补齐透明度曲线单测。
