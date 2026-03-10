# Viewer Agent 模块可见渲染设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-agent-module-rendering.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-agent-module-rendering.project.md`

## 1. 设计定位
定义 Agent 从单球体升级为更接近机器人形态的渲染结构，通过模块立方体拼接与 `height_cm` 对齐的尺寸映射，让观察者一眼看出模块数量和体积差异。

## 2. 设计结构
- 主体形态层：用胶囊体替代球体，严格按 `height_cm` 推导主体高度与半径。
- 模块表达层：从 `body_state.slots` 读取模块数量，渲染为小立方体拼接布局。
- 调试支持层：`AgentMarker` 携带 `module_count` 便于断言与测试。
- 兼容层：保持选中、高亮与标签机制不变。

## 3. 关键接口 / 入口
- `Agent.body.height_cm`
- `Agent.body_state.slots[*].installed_module`
- `Viewer3dAssets`
- `AgentMarker.module_count`
- `spawn_agent_entity`

## 4. 约束与边界
- 模块可见化不能引入复杂骨骼动画或外部高模资产。
- 高度映射需严格可解释，宽高比使用固定比例并加上下限/上限。
- 模块立方体数量需有上限，避免极端 draw call 激增。
- 首版以快照数据为准，事件级精细同步后续再补。

## 5. 设计演进计划
- 先完成胶囊体主体与尺寸映射。
- 再接入模块数量读取与立方体布局。
- 最后通过截图、测试和调试字段收口 Agent 渲染改造。
