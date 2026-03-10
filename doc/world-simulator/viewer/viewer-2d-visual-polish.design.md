# Viewer 2D 可视化精修设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-2d-visual-polish.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-2d-visual-polish.project.md`

## 1. 设计定位
定义 2D 观察态的地图可读性增强方案，通过符号层、标签 LOD 与平面化流向覆盖层，让 2D 模式在不影响 3D 表达的前提下具备更强的信息提取效率。

## 2. 设计结构
- 地图符号层：为 Location/Agent 提供 2D 专用高对比符号。
- 标签策略层：按 2D 模式单独配置标签距离衰减、容量与遮挡阈值。
- 流向覆盖层：在 2D 视角使用平面化流线与方向箭头强化路径可读性。
- 模式切换层：2D 显示符号层，3D 隐藏，保证双模式职责清晰。

## 3. 关键接口 / 入口
- 2D 专用 marker 组件
- `ViewerCameraMode::TwoD / ThreeD`
- `update_label_lod`
- 平面化流向覆盖层与箭头头部

## 4. 约束与边界
- 精修只针对 2D 可读性，不改变 3D 交互与风格。
- 不修改协议、事件模型与世界快照结构。
- 2D 专用元素必须随相机模式稳定切换，不残留到 3D。
- 本轮不引入编辑器式交互。

## 5. 设计演进计划
- 先建立 2D 符号层与模式同步。
- 再细化标签 LOD 和流向覆盖层表现。
- 最后以截图闭环与测试固定 2D 精修效果。
