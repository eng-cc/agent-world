# Viewer 文本可选中与复制能力设计

## 目标
- 让 `agent_world_viewer` 中关键信息文本支持“鼠标选中 + 系统复制快捷键（Cmd/Ctrl+C）”。
- 在不破坏既有 Bevy UI 布局和交互（时间轴、控制按钮、选中联动）的前提下，提供稳定可复制入口。
- 保持中文/英文双语一致体验，避免新增硬编码英文提示。
- 保持现有 UI 文本生成链路不变，降低回归风险。

## 范围
- **范围内**
  - `agent_world_viewer` 引入 `bevy_egui` 可选中文本面板。
  - 面板展示并同步以下现有 UI 文本：状态、当前选择、世界摘要、Agent 活动、选中详情、事件列表、诊断结论、事件联动、时间轴状态、覆盖层状态。
  - Top Controls 区新增“显示/隐藏复制面板”开关按钮（中英文文案）。
  - 新增/更新单元测试，覆盖开关按钮行为与语言切换文案刷新。
- **范围外**
  - 不改造 Bevy 原生 `Text` 组件为可编辑输入框。
  - 不新增“导出到文件/剪贴板按钮”离线导出功能。
  - 不引入历史快照缓存、全文检索等增强功能。

## 接口 / 数据

### 1) UI 状态资源
- 新增 `CopyableTextPanelState`（Bevy Resource）：
  - `visible: bool`：复制面板是否显示。
  - 默认值：`true`（开箱可用）。

### 2) 交互接口
- Top Controls 新增按钮：
  - 点击后在 `visible=true/false` 间切换。
  - 按钮文本根据语言和当前状态动态变化（Hide/Show）。

### 3) 文本同步方式
- 复制面板不自行重算业务文本，直接读取现有 UI `Text` 组件内容。
- 读取目标组件：
  - `StatusText`、`SelectionText`、`SummaryText`、`AgentActivityText`、`SelectionDetailsText`、`EventsText`
  - `DiagnosisText`、`EventObjectLinkText`、`TimelineStatusText`、`WorldOverlayStatusText`
- 展示控件：`egui::Label::selectable(true)`，支持拖选并使用系统快捷键复制。

### 4) 调度与依赖
- `run_ui` 注册 `EguiPlugin`。
- 在 `EguiPrimaryContextPass` 中绘制复制面板，避免与主 Update 逻辑耦合。
- 复制面板仅用于 UI 模式；headless 模式保持不变。

## 里程碑
- **M1 文档阶段**：完成设计文档与项目管理文档。
- **M2 代码阶段**：接入 `bevy_egui`，实现复制面板与可见性开关。
- **M3 测试阶段**：补充开关与文案回归测试，执行 `cargo check/test`。
- **M4 收口阶段**：更新项目管理文档状态与当日 devlog。

## 风险
- **UI 覆盖风险**：新增浮层可能遮挡既有右侧信息区。
  - 缓解：提供显式开关按钮，默认可隐藏。
- **输入冲突风险**：复制面板鼠标交互与 3D 相机拖拽潜在冲突。
  - 缓解：面板放置在右侧并依赖 `bevy_egui` 输入接管，保持 3D 视口交互边界。
- **文案漂移风险**：新增按钮未接入 i18n 时可能出现单语回归。
  - 缓解：按钮文案统一进入 `i18n.rs` 并补充测试。
