# Viewer 发行体验沉浸改造 Phase 6 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase6.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase6.project.md`

## 1. 设计定位
定义第六阶段的任务驱动与电影化首屏方案：用短时开场、任务追踪 HUD、奖励反馈和小地图导向，把 Player 模式推进到连续可玩的发行体验。

## 2. 设计结构
- 电影化开场层：根据连接态与世界 Tick 计算短时叙事覆盖与透明度曲线。
- 任务驱动层：复用 `Connect -> OpenPanel -> Select -> Explore` 闭环输出完成数、当前步骤和下一动作提示。
- 奖励反馈层：对任务推进与完成态提供更明确的文本和颜色层级。
- 空间导向层：用小地图卡片展示位置缩略与当前选中高亮。

## 3. 关键接口 / 入口
- `ViewerState.status/snapshot/events/metrics`
- `ViewerSelection.current`
- `RightPanelLayoutState.panel_hidden`
- `egui_right_panel_player_experience.rs` / `egui_right_panel_player_guide.rs`

## 4. 约束与边界
- 所有新增视觉层必须低遮挡、低频、强可读。
- 任务状态继续复用既有 guide/progress snapshot，不引入第二套状态机。
- 小地图优先承担方向感辅助，不扩展成完整战术地图系统。
- Director 模式调试布局保持不变，避免影响现有开发链路。

## 5. 设计演进计划
- 先落电影化开场与透明度窗口。
- 再补任务 HUD、奖励反馈和小地图。
- 最后通过 Web 闭环产物验证连续体验链路。
