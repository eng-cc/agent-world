# Viewer Web 全屏与面板总开关设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-fullscreen-panel-toggle.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-fullscreen-panel-toggle.project.md`

## 1. 设计定位
定义 Web 端全屏自适应和右侧面板整体显隐方案：让浏览器可用区域被最大化利用，并在隐藏主面板时即时释放 3D 视口空间。

## 2. 设计结构
- 窗口自适应层：wasm 路径下窗口尺寸跟随浏览器父容器/viewport。
- 宽度策略层：主面板与 Chat 面板按可用宽度动态计算，而非固定像素上限。
- 总开关层：新增右侧区域整体隐藏/显示控制。
- 命中边界层：隐藏主面板后把 `RightPanelWidthState` 置零，更新 3D 输入边界。

## 3. 关键接口 / 入口
- wasm 窗口配置
- `RightPanelWidthState`
- 主面板/Chat 面板宽度策略
- 右侧面板总开关

## 4. 约束与边界
- 不改 3D 材质与场景渲染逻辑。
- 不增加跨端持久化，本轮只用内存态。
- 模块级开关与 Chat 独立面板能力要保留。
- 隐藏面板后 3D 命中边界必须即时扩展，不能有残留遮挡区。

## 5. 设计演进计划
- 先实现 wasm 全屏自适应。
- 再补动态宽度和面板总开关。
- 最后验证面板显隐与 3D 边界联动。
