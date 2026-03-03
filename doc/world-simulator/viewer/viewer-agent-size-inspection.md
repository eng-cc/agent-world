# Viewer Agent 尺寸可观测与比例校验

## 目标
- 回答并可视化验证「Agent 与 Location 的比例是否符合数据口径」。
- 在右侧选中详情中，点击 Agent 时直接显示其尺寸信息，减少来回查代码/查快照。
- 保持现有渲染行为不变，只增强可观测性，避免引入额外渲染回归。

## 范围
### 范围内
- `agent_world_viewer` 右侧 Selection Details 的 Agent 分支新增尺寸字段。
- 尺寸信息至少包含：
  - Agent 身高（cm / m）
  - 所在 Location 半径（cm / m，若可用）
  - Agent 身高与 Location 半径比例（height / location_radius）
- 补充单元测试，覆盖尺寸字段渲染与格式。
- 补充中文文案本地化映射（若新增英文标签）。

### 范围外
- 不修改 simulator 数据模型（`height_cm` / `radius_cm` 语义不变）。
- 不修改 3D 实体几何参数（胶囊体比例、模块 marker 布局保持不变）。
- 不引入新的面板或交互流程（复用现有 Selection Details 区域）。

## 接口 / 数据
- 输入数据来源：
  - `WorldSnapshot.model.agents[agent_id].body.height_cm`
  - `WorldSnapshot.model.locations[agent.location_id].profile.radius_cm`（若存在）
- 新增展示字段（Selection Details / Agent）：
  - `Body Size: data_height={m}m ({cm}cm)`
  - `Location Radius: {cm}cm ({m}m)`
  - `Scale Ratio: height/location_radius={ratio}`
- 降级策略：
  - 无快照/无 Agent：保持现有降级文案。
  - Agent 所在 Location 不存在：仅展示 Agent 尺寸，不展示 Location 比例行。

## 里程碑
- **ASI-1**：设计文档与项目管理文档落地。
- **ASI-2**：Selection Details 增加 Agent 尺寸与比例展示。
- **ASI-3**：补充测试与 i18n 映射，执行回归测试并收口。

## 风险
- 若场景数据默认 `Location.radius_cm` 偏小，比例输出会直观暴露“Agent 看起来不小”的问题；这是数据事实而非 UI bug，需要避免误判。
- 新增文案需同步中文替换规则，否则中英文混排可读性下降。
- 文本行数增加后需关注右侧滚动体验，但当前已有滚动容器可承载。
