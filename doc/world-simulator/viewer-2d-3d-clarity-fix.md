# Viewer 2D/3D 可视化清晰度修复设计

## 目标
- 修复当前 Viewer 在 `llm_bootstrap` 等场景中出现的“2D 大块遮屏、3D 首屏不可读”问题。
- 统一场景实体的尺寸单位语义，避免“位置按 world unit、尺寸按米”的比例错位。
- 降低首次进入时右侧信息区认知负担，让用户先看到“可读主画面”。

## 范围

### In Scope
- Location 渲染尺寸改为基于 `cm_to_unit` 的 world unit 尺度映射，并引入可视化上下限。
- 修复 2D 自动聚焦链路：`orbit.radius` 变化后同步正交投影 `scale`。
- 调整右侧模块默认可见状态，收敛首次进入时的信息密度。
- 补齐相关单测与回归截图闭环。

### Out of Scope
- 不改 world 协议与 snapshot/event 数据结构。
- 不重做材质体系与 2D/3D 视觉风格。
- 不新增复杂交互（编辑器、拖拽建造等）。

## 接口 / 数据

### 1) Location 渲染尺度
- 保留 `location.profile.radius_cm` 作为世界语义数据。
- 新增渲染层换算函数（草案）：
  - 输入：`radius_cm`、`cm_to_unit`
  - 输出：`radius_world_units`
  - 口径：`radius_cm * cm_to_unit`，并在渲染层做最小/最大可见钳制。
- 仅影响可视化 `Transform::scale` 与标签偏移，不影响内核物理与资源结算。

### 2) 2D 自动聚焦投影同步
- `auto_focus` 在 2D 模式写入 `orbit.radius` 后，立即同步 `OrthographicProjection.scale`。
- 保持现有 3D 强制聚焦逻辑不变。

### 3) 右侧默认可见模块
- 默认保留核心区块（`controls/overview/event_link`）。
- 默认收起高密度区块（`overlay/diagnosis/timeline/details`），用户可按需打开。
- 仅调整默认值，不改模块开关机制与持久化文件格式。

## 里程碑
- **CFX1**：文档与任务拆解（本轮）。
- **CFX2**：完成 Location 尺度修复与测试。
- **CFX3**：完成 2D 自动聚焦投影同步与测试。
- **CFX4**：完成默认模块可见性收敛、截图回归与文档收口。

## 风险
- 小尺寸 Location 过度缩小导致不可见。
  - 缓解：设置渲染最小半径钳制，并补充极小半径单测。
- 自动聚焦改动影响 3D 行为。
  - 缓解：2D/3D 分支显式区分，保留既有 3D 路径。
- 默认区块收起引发“功能被隐藏”反馈。
  - 缓解：保持模块开关入口显著，默认只收起非核心区块。
