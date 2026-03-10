# Viewer Frag 比例与选中稳定性设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-frag-scale-selection-stability.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-frag-scale-selection-stability.project.md`

## 1. 设计定位
定义 frag 实际比例、选中显示与相机口径稳定化方案：让 frag 尺度始终与 snapshot 数据量纲一致，并在选中/取消选中、缩放与聚焦过程中保持可见且不失真。

## 2. 设计结构
- 尺度映射层：`radius_cm * cm_to_unit` 作为统一线性口径，移除破坏比例的可视化钳制。
- 高亮恢复层：为 frag 分块实体补齐 `BaseScale`，保证高亮复位后仍回到真实比例。
- 选中表现层：`SelectionKind::Fragment` 禁用黄色 halo，仅保留不改变尺寸的选中反馈。
- 相机适配层：near/far 裁剪、最小缩放半径与 focus 半径统一按世界单位推导。

## 3. 关键接口 / 入口
- `scene_helpers::location_render_radius_units`
- `location_fragment_render::spawn_location_fragment_elements`
- `selection_emphasis.rs`
- `camera_controls` / 自动聚焦相关测试

## 4. 约束与边界
- 不改 snapshot 协议与 frag 业务数据，只修正 Viewer 侧尺度与选中表现。
- frag 去 halo 的同时仍要保留足够的选中可感知性，不能退化成交互不可见。
- 相机口径修复必须与真实量纲一致，不能再靠经验常数兜底。
- 后续若新增实体类型复用高亮链路，同样需要维护 `BaseScale`。

## 5. 设计演进计划
- 先冻结线性尺度与高亮恢复口径。
- 再修 frag 选中表现与 Agent/相机量纲一致性。
- 最后通过定向测试和日志回写收口稳定性修复。
