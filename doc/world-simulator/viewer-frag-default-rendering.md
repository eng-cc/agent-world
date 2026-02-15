# Agent World Simulator：Viewer 默认渲染 Frag 且不渲染 Location（设计文档）

## 目标
- Viewer 场景中不再渲染 location 外层几何与标签，降低遮挡与视觉噪声。
- 移除 frag 渲染开关，默认始终渲染 frag（fragment blocks）。
- 在 frag 选中详情中仅展示其所属 location，避免冗余信息。

## 范围

### In Scope
- 去除 `WorldOverlayConfig` 中对 frag 渲染的布尔开关与环境变量入口。
- 场景刷新路径改为“默认渲染 frag block”，不依赖 UI flag。
- location 仅保留逻辑锚点（位置/半径/选择数据），不渲染 mesh/label。
- 新增 frag 选择标记与详情文案：只显示所属 location。
- 更新/补充对应单元测试（渲染配置、选择详情、场景刷新路径）。

### Out of Scope
- 不改动 world server 协议与 snapshot 数据结构。
- 不新增 frag 与非 frag location 的业务关系推导（仅使用已有所属 location 上下文）。
- 不调整经济/物理仿真参数。

## 接口 / 数据
- 渲染配置：
  - 删除 `WorldOverlayConfig.show_fragment_elements`。
  - 删除环境变量 `AGENT_WORLD_VIEWER_SHOW_FRAGMENT_ELEMENTS` 解析。
- 场景构建：
  - `scene_helpers` / `scene_dirty_refresh` 默认尝试渲染 `location.fragment_profile.blocks`。
  - location 实体仅作为锚点保留（`LocationMarker + Transform + BaseScale`），不附加可见几何。
- 选择与详情：
  - 增加 frag 选择标记（记录 `location_id` 与 block 索引）。
  - 详情面板新增 frag 分支，仅输出“所属 location”。

## 里程碑
- **FDR1**：完成设计文档与项目管理文档。
- **FDR2**：完成“location 不渲染 + frag 默认渲染 + 移除开关”代码改造。
- **FDR3**：完成 frag 选择/详情与测试回归。
- **FDR4**：更新使用文档与 devlog 收口。

## 风险
- 去掉开关后，frag 数量较大场景可能带来性能回退。
- location 不可视后，用户对空间锚点理解可能下降，需要依赖详情/选中信息补偿。
- 选择逻辑新增 frag 分支后，需防止与既有 location/asset 选中优先级冲突。
