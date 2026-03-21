# Viewer 3D 可视化架构设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-visualization-3d.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-visualization-3d.project.md`

## 1. 设计定位
定义 oasis7 在 Bevy 中的 3D 可视化最小闭环：让 3D 场景承担空间态势直读，右侧面板承担语义与因果解释。

## 2. 设计结构
- 数据驱动层：复用 `WorldSnapshot` + `WorldEvent` 初始化并增量更新 3D 世界。
- 坐标映射层：`GeoPos` 按 `cm_to_unit` 转为 Bevy 坐标，并处理轴约定。
- 实体表达层：Location/Agent 建立统一实体索引、标签和选中高亮。
- 交互分区层：3D viewport 与右侧 UI 切分，相机控制和拾取共用同一空间语义。

## 3. 关键接口 / 入口
- `Viewer3dConfig`
- `WorldSnapshot` / `WorldEvent`
- `GeoPos -> Vec3` 映射
- 相机控制 / 拾取 / UI 选中同步

## 4. 约束与边界
- 本阶段只做最小 3D 闭环，不引入复杂模型资产、光照系统和大规模优化。
- Viewer 协议保持不变，3D 只是新的消费层。
- 3D 视图优先承担空间信息，语义解释仍由右侧面板完成。
- 对象选中后尽量单屏给出关键状态与上下文。

## 5. 设计演进计划
- 先落 3D 场景骨架与坐标映射。
- 再补 snapshot/event 驱动与相机交互。
- 最后通过拾取和 UI/3D 分区收口最小闭环。
