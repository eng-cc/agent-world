# Viewer WASD 相机导航设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-wasd-camera-navigation.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-wasd-camera-navigation.project.md`

## 1. 设计定位
定义 2D/3D 共用的 WASD 相机平移方案：降低只靠鼠标拖拽的操作成本，同时不破坏既有旋转、缩放、自动聚焦和 EGUI 输入。

## 2. 设计结构
- 输入轴层：`W/A/S/D` 归一为前后/左右两个平移轴。
- 相机平移层：仅更新 `OrbitCamera.focus`，不修改 yaw/pitch/radius。
- 方向投影层：按相机朝向投影到水平面，避免 pitch 带来垂直漂移。
- 冲突保护层：EGUI 键盘占用或非 3D 视口命中时禁用 WASD 移动。

## 3. 关键接口 / 入口
- `ButtonInput<KeyCode>`
- `OrbitCamera.focus`
- 相机前向/右向 XZ 投影
- EGUI 键盘占用检测

## 4. 约束与边界
- 不引入飞行模式、Q/E 升降或第一人称控制。
- 不改相机缩放/旋转口径和协议模型。
- WASD 必须和现有鼠标交互边界一致，不能抢占文本输入。
- 速度需随 `orbit.radius` 联动，避免远近缩放体感割裂。

## 5. 设计演进计划
- 先补输入轴和 focus 平移。
- 再做 2D/3D 共用与键盘冲突保护。
- 最后通过 camera_controls 回归收口导航体验。
