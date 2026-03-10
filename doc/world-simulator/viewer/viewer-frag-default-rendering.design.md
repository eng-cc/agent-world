# Viewer Frag 默认渲染与 Location 去可视化设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-frag-default-rendering.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-frag-default-rendering.project.md`

## 1. 设计定位
定义 Viewer 默认渲染 frag、同时移除 location 外层可视几何的方案：通过减少空间噪声与遮挡，让玩家直接看到真实可交互的 fragment block，并保留 location 作为逻辑锚点。

## 2. 设计结构
- 渲染配置层：删除 frag 显示开关与环境变量入口，固定默认渲染策略。
- 场景构建层：`scene_helpers` 与 `scene_dirty_refresh` 始终尝试渲染 `location.fragment_profile.blocks`。
- 锚点保留层：location 仅保留位置、半径和选择上下文，不再输出可见 mesh/label。
- 选择详情层：新增 frag 选择分支，详情面板仅展示所属 location，避免冗余信息。

## 3. 关键接口 / 入口
- `WorldOverlayConfig`
- `scene_helpers.rs` / `scene_dirty_refresh.rs`
- `location_fragment_render.rs`
- `selection_linking.rs` / `egui_right_panel.rs`

## 4. 约束与边界
- 不改 world snapshot 协议与 fragment 业务语义，只调整 Viewer 渲染默认值。
- location 去可视化后仍需保留逻辑锚点，不能影响选择、定位和上下文关联。
- frag 选择新增后要避免与现有 location/asset 选中优先级冲突。
- 默认渲染可能带来性能压力，后续优化应在不恢复旧开关的前提下处理。

## 5. 设计演进计划
- 先清理 frag 开关与 location 可视输出。
- 再补 frag 选择详情与相关测试。
- 最后通过说明文档和日志收口新的默认渲染语义。
