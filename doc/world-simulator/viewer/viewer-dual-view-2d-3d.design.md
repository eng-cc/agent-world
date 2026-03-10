# Viewer 双视角 2D/3D 切换设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-dual-view-2d-3d.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-dual-view-2d-3d.project.md`

## 1. 设计定位
定义 Viewer 的 2D 全局视角与 3D 透视视角双模式：默认进入 2D 提高开箱可观察性，并通过右侧按钮在两种视角间稳定切换。

## 2. 设计结构
- 模式资源层：`ViewerCameraMode` 管理 `TwoD/ThreeD`。
- 相机同步层：初始化与模式切换时写入对应的轨道/俯视预设。
- UI 入口层：右侧顶部提供 2D/3D 双向切换按钮。
- 视觉辅助层：提升 Agent 默认颜色，改善复杂背景上的识别度。

## 3. 关键接口 / 入口
- `ViewerCameraMode`
- `camera_controls.rs`
- `egui_right_panel.rs`
- `i18n.rs`
- 2D/3D 切换按钮

## 4. 约束与边界
- 2D 模式必须显式禁用旋转并保持俯视稳定。
- 切换预设要优先保证“全局可见”和“可控”，允许有轻微跳变。
- 本轮不重构右侧信息架构或引入新底图。
- world floor/bounds 的干扰背景需在双视角中统一处理。

## 5. 设计演进计划
- 先建立 `ViewerCameraMode` 和默认 2D 视角。
- 再接入按钮与相机同步逻辑。
- 最后调整 Agent 颜色与背景面显示策略，完成双视角收口。
