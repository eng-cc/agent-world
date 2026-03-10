# Viewer 3D 精致化与性能优化设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-3d-polish-performance.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-3d-polish-performance.project.md`

## 1. 设计定位
定义 Viewer 在 3D 精致化与性能预算之间的平衡方案，通过渲染质量分档、标签与覆盖层 LOD、性能摘要和脏区刷新，把视觉升级建立在可量化预算之上。

## 2. 设计结构
- 质量分档层：`Debug / Balanced / High` 管理阴影、标签、覆盖层与线渲染模式。
- 视觉分层层：整理对象层级、选中反馈、标签显示与遮挡降权。
- 性能预算层：采样 `frame ms`、对象数、标签数与覆盖层成本，形成预算摘要。
- 更新策略层：以脏区刷新和节流降低高频 UI/覆盖层开销。

## 3. 关键接口 / 入口
- `ViewerRenderQualityConfig`
- `ViewerPerfBudget`
- `line_render_mode`
- `world_overlay.rs`
- `egui_right_panel.rs`

## 4. 约束与边界
- 性能优化优先通过编译边界、LOD 与更新策略实现，不破坏协议主结构。
- 视觉增强不能让信息噪声高于现状。
- 批处理与优化不能影响拾取、选中和时间轴联动语义。
- 本阶段不引入云渲染或跨客户端协同渲染。

## 5. 设计演进计划
- 先建立性能基线与质量分档。
- 再推进视觉精致化与覆盖层/标签优化。
- 最后将性能摘要接入 UI，并用截图与基线对比收口。
