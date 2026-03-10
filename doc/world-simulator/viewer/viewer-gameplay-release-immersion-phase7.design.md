# Viewer 发行体验沉浸改造 Phase 7 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase7.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase7.project.md`

## 1. 设计定位
定义第七阶段的整体布局重构与指挥优先方案：让 Player 默认保持世界优先的隐藏态，同时保证 Chat/Command 指挥链路在任意时刻都直接可达。

## 2. 设计结构
- 布局预设层：围绕任务/指挥/情报三种 Player 预设组织默认信息架构。
- 直接指挥层：隐藏面板状态下提供显式入口，一步切到指挥态。
- 默认可见性层：按 Player 预设重排 chat、overview、event_link 等模块显隐。
- 宽度预算层：收紧右侧面板默认宽度，避免持续挤占世界视野。

## 3. 关键接口 / 入口
- `RightPanelLayoutState`
- `RightPanelModuleVisibilityState`
- `egui_right_panel_player_entry.rs`
- `ViewerState` / `ViewerSelection`

## 4. 约束与边界
- 只重构 Viewer 布局与交互，不改协议、事件语义和后端接口。
- 三个预设必须语义稳定、切换可预测，避免制造新认知负担。
- 预设不能破坏既有 guide step 判定，只改可见性与入口组织。
- 隐藏态下的指挥入口必须在 Web 闭环中稳定可达。

## 5. 设计演进计划
- 先冻结三种布局预设与宽度预算。
- 再补隐藏态直接指挥入口和模块联动。
- 最后通过回归与截图验证世界优先布局。
