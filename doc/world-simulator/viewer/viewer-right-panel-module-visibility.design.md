# Viewer 右侧面板模块可见性设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-right-panel-module-visibility.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-right-panel-module-visibility.project.md`

## 1. 设计定位
定义右侧面板模块化显隐与交互密度控制方案：让不同信息区块按模块状态稳定展示，并在窄宽度、长内容和加载态下保持可读。

## 2. 设计结构
- 模块显隐层：按 overview、event_link、timeline、diagnosis 等模块状态控制可见性。
- 滚动分区层：区分顶部固定区与内容区，避免上下区域串滚。
- 交互反馈层：控制按钮 hover/pressed、`Step ...` pending 等状态统一反馈。
- 布局收敛层：统一间距、边框、换行和顶部折叠行为。

## 3. 关键接口 / 入口
- 右侧面板模块可见性状态
- `ComputedNode` 命中边界
- `Step ...` pending 反馈
- `Hide/Show Top` 顶部折叠入口

## 4. 约束与边界
- 只调面板结构和显隐策略，不改 world 协议与主交互语义。
- 长详情和长事件列表必须始终可滚动访问。
- 高 DPI 坐标和真实 UI 边界要统一，否则滚动命中会漂移。
- 顶部折叠与模块显隐联动需可预测，不能出现随机覆盖。

## 5. 设计演进计划
- 先修面板重叠和换行问题。
- 再补滚动分区与按钮反馈。
- 最后接顶部折叠与视觉样式收口。
