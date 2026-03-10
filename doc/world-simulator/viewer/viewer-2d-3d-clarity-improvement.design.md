# Viewer 2D/3D 可视化清晰度修复设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-2d-3d-clarity-improvement.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-2d-3d-clarity-improvement.project.md`

## 1. 设计定位
定义 Viewer 在 2D/3D 双模式下的首屏可读性修复方案：统一 Location 尺度换算、修正 2D 自动聚焦投影同步，并降低首次进入的右侧信息密度。

## 2. 设计结构
- 尺度换算层：`radius_cm` 通过 `cm_to_unit` 转成 world unit，并在渲染层做可见性钳制。
- 2D 聚焦层：`orbit.radius` 更新后同步正交投影 `scale`，防止首屏失真。
- 信息密度层：右侧仅默认展示核心区块，其余高密度模块按需展开。
- 回归层：通过单测与截图闭环固定 2D/3D 清晰度基线。

## 3. 关键接口 / 入口
- `scene_helpers.rs`
- `auto_focus.rs`
- `camera_controls.rs`
- `right_panel_module_visibility.rs`
- `radius_cm -> world units`

## 4. 约束与边界
- 尺度修复只影响渲染层，不改世界协议与物理结算语义。
- 2D 修复不能破坏既有 3D 聚焦路径。
- 默认模块收敛只改默认可见状态，不改开关机制与持久化格式。
- 极小半径对象仍需保持最小可见性。

## 5. 设计演进计划
- 先固定尺度换算和聚焦同步逻辑。
- 再收敛首屏右侧模块默认可见性。
- 最后通过截图回归和测试验证清晰度修复稳定。
