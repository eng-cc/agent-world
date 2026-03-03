# Agent World Simulator：Viewer Location 开采损耗可视化（设计文档）

## 目标
- 在 Viewer 中让“location 被开采后体量下降”可被直观看到，而不只停留在数值面板。
- 对 `asteroid fragment` 场景提供稳定的损耗反馈，便于人工观察与截图回归。
- 保持与现有模拟器协议兼容，不新增 server 事件字段。

## 范围

### In Scope
- 基于 `location.fragment_budget` 的剩余占比，动态计算渲染半径缩放。
- 将缩放应用到 location 主球体与细节子节点（ring/halo）链路。
- 在详情文本中补充损耗比例/剩余质量指标，帮助数值与图形对齐。
- 增加单元测试覆盖关键计算与渲染行为。

### Out of Scope
- 不实现真实布尔切割/体素挖洞等高成本几何重建。
- 不新增平滑动画插值（先以离散快照更新为主）。
- 不改动 `agent_world` 的事件协议和世界状态结构。

## 接口 / 数据
- 输入来源：`WorldSnapshot.model.locations[*].fragment_budget`
  - `total_by_element_g`
  - `remaining_by_element_g`
- 缩放策略：
  - 计算剩余质量比例 `remaining_ratio = remaining_total / total_total`
  - 体积到半径映射采用立方根：`radius_factor = cbrt(remaining_ratio)`
  - 设最小可视因子 `min_radius_factor` 避免实体完全消失。
- 详情文本新增：
  - `Fragment Depletion: <mined%> (remaining/total)`

## 里程碑
- **LDV1**：设计文档与任务拆解。
- **LDV2**：viewer 渲染接入 budget->半径缩放。
- **LDV3**：详情文本补充损耗指标 + 单测回归。

## 风险
- 极端低剩余时若半径过小，可能导致选取困难；通过最小可视因子兜底。
- 快照频率较低时，损耗反馈会呈阶梯变化；后续可考虑动画平滑。
- 若 location 无 `fragment_budget`，需安全回退为原半径渲染。
