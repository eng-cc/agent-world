# Viewer Web 全屏自适应与右侧面板整体显隐设计

## 目标
- 让 `agent_world_viewer` 在 Web 端默认占满浏览器可用区域，避免固定窗口尺寸导致的显示受限。
- 让右侧综合面板支持“一键整体隐藏/显示”，隐藏后让 3D 视口可用区域最大化。
- 保持现有模块级开关、Chat 独立面板、3D 输入命中边界逻辑不退化。

## 范围
- 范围内：
  - Web 端窗口配置改为跟随浏览器父容器尺寸（full viewport）。
  - 右侧主面板与 Chat 面板宽度策略从固定像素上限调整为按可用宽度动态计算。
  - 新增右侧面板总开关：支持隐藏/显示整个右侧区域。
  - 面板隐藏时将 `RightPanelWidthState` 置为 `0`，使 3D 交互区域即时扩展。
- 范围外：
  - 不改动 3D 场景渲染内容与材质逻辑。
  - 不改动 viewer/live 协议字段。
  - 不新增跨端持久化（本任务仅使用现有内存布局状态）。

## 接口 / 数据

### 1) Web 窗口配置
- 文件：`crates/agent_world_viewer/src/app_bootstrap.rs`
- 在 wasm32 路径下启用 `Window.fit_canvas_to_parent = true`，保持 `prevent_default_event_handling = false`。

### 2) 右侧面板布局状态扩展
- 文件：`crates/agent_world_viewer/src/panel_layout.rs`
- 扩展 `RightPanelLayoutState`：
  - `panel_hidden: bool`（默认 `false`）
- 语义：
  - `panel_hidden = true` 时，不渲染主面板和 Chat 面板，仅保留一个悬浮“显示面板”按钮。

### 3) 右侧面板渲染与宽度
- 文件：`crates/agent_world_viewer/src/egui_right_panel.rs`
- 新增总开关按钮文案（复用 i18n）：
  - 面板显示时：`Hide Panel / 隐藏面板`
  - 面板隐藏时：`Show Panel / 显示面板`
- 宽度策略：
  - 主面板、Chat 面板采用“最小宽度 + 动态最大宽度（按可用宽度比例）”。
  - 不再使用固定像素上限常量。
- 面板隐藏时：
  - `panel_width.width_px = 0.0`
  - 3D 视口/输入命中逻辑自动复用既有 `RightPanelWidthState`。

## 里程碑
- M1：设计文档与项目管理文档完成。
- M2：实现 Web 全屏自适应与右侧面板总开关。
- M3：补充/更新单测，完成 `agent_world_viewer` 与 wasm 编译验证。
- M4：更新使用手册、任务日志与项目状态收口。

## 风险
- 风险 1：面板隐藏后入口丢失，用户无法恢复。
  - 缓解：隐藏状态下保留固定位置悬浮“显示面板”按钮。
- 风险 2：动态宽度策略导致极小窗口下布局拥挤。
  - 缓解：保留面板最小宽度并限制动态上限比例。
- 风险 3：面板宽度状态变化影响 3D 输入边界。
  - 缓解：继续复用统一的 `RightPanelWidthState`，并补行为测试。
