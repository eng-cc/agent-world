# Viewer 3D 渲染物理准确性设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-rendering-physical-accuracy.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-rendering-physical-accuracy.project.md`

## 1. 设计定位
定义 Viewer 3D 渲染与世界物理数据对齐的统一口径：让尺寸、距离、光照、材质和热/辐射可视化都以真实量纲为基础，而不是依赖经验缩放。

## 2. 设计结构
- 量纲映射层：固定 `1 world unit = 1 meter`，统一 cm→m 转换。
- 大场景精度层：CPU 保留 `f64` 世界坐标，GPU 提交前采用 floating origin。
- 几何与 LOD 层：Location/Fragment/Agent 都以真实半径或高度驱动几何尺寸和可见性策略。
- 光照材质层：以小行星带辐照度、物理材质参数和热/辐射阈值建立显示基线。

## 3. 关键接口 / 入口
- `GeoPos(x_cm,y_cm,z_cm)`
- `floating origin` / `render_origin_m`
- `stellar_distance_au` / `irradiance_w_m2`
- 物理材质参数库与热态可视口径

## 4. 约束与边界
- 不修改 simulator 内核物理规则，只定义 viewer 显示口径。
- 不引入高精度天体力学或复杂资产管线。
- 真实尺寸不足时优先用轮廓描边等视觉补偿，而不是放大模型本体。
- 远景对象要通过标记层兜底，不能强行拉大 far plane 破坏精度。

## 5. 设计演进计划
- 先冻结统一单位和精度策略。
- 再接几何、光照、材质和热态映射。
- 最后通过尺寸/距离/曝光回归验证物理一致性。
