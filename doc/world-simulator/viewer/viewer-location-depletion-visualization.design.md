# Viewer Location 开采损耗可视化设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-location-depletion-visualization.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-location-depletion-visualization.project.md`

## 1. 设计定位
定义基于 `fragment_budget` 的 location 开采损耗可视化方案：用剩余质量比例驱动半径缩放，并在详情面板中补足损耗指标，帮助观察资源枯竭进程。

## 2. 设计结构
- 数据输入层：读取 `fragment_budget.total_by_element_g` 与 `remaining_by_element_g`。
- 缩放映射层：以 `cbrt(remaining_ratio)` 计算半径因子，并设置最小可视兜底。
- 详情表达层：输出 `Fragment Depletion` 损耗文本和 remaining/total 指标。
- 安全回退层：无 budget 时继续使用原半径渲染。

## 3. 关键接口 / 入口
- `fragment_budget`
- `scene_helpers.rs`
- `ui_text.rs`
- `remaining_ratio` / `min_radius_factor`

## 4. 约束与边界
- 不改内核碎片生成和物理规则，只增强 Viewer 表达。
- 极低剩余量时仍需保持最小可视性，不能彻底消失。
- 快照阶梯变化属于数据频率限制，本轮不做插值动画。
- 详情文本与几何缩放需共用同一预算口径，避免显示自相矛盾。

## 5. 设计演进计划
- 先接 fragment budget 到半径缩放。
- 再补详情损耗指标和回退逻辑。
- 最后通过渲染/文本测试收口开采损耗表达。
