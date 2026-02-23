# Viewer 发行体验改造（第四阶段：项目管理文档）

## 任务拆解
- [x] VRI4P0：建立第四阶段设计文档与项目管理文档。
- [x] VRI4P1：实现 Player 场景氛围层（背景层次/呼吸光晕/边缘发光）。
- [x] VRI4P2：实现 Player 引导与目标卡片过渡动效（淡入/位移/脉冲）。
- [ ] VRI4P3：执行回归与 Web 闭环验收并完成文档收口。

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_card_motion.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VRI4P2 完成，VRI4P3 进行中。
- 阻塞项：无。
- 最近更新：引导卡/目标卡已接入淡入+位移+脉冲过渡，进入回归与 Web 闭环验收。
