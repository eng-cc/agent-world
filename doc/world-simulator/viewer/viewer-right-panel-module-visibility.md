# Viewer 右侧面板模块开关与本地缓存设计

## 目标
- 为 `agent_world_viewer` 右侧 `egui` 面板中的每个业务模块提供独立开关，支持按需隐藏。
- 将模块开关状态缓存到本地文件，重启 viewer 后保持用户上一次偏好。
- 保持现有功能逻辑（控制、联动、时间轴、覆盖层）不退化，不影响 headless 模式。

## 范围
- **范围内**
  - 在右侧面板新增“模块开关”区域，覆盖以下模块的显示控制：
    - 控制按钮（Play/Pause/Step/Seek）
    - 状态总览
    - 覆盖层
    - 诊断
    - 事件联动
    - 时间轴
    - 状态明细（含事件行）
  - 新增本地缓存文件读写能力：启动加载、交互变更后落盘。
  - 补充单元测试：默认值、序列化兼容、开关可见性联动。
- **范围外**
  - 不调整模块内部业务逻辑与文案内容。
  - 不引入云端同步或多设备共享配置。
  - 不修改 viewer/server 协议。

## 接口 / 数据

### 1) 模块开关状态资源
- 新增 `RightPanelModuleVisibilityState`（Resource）：
  - `show_controls`
  - `show_overview`
  - `show_overlay`
  - `show_diagnosis`
  - `show_event_link`
  - `show_timeline`
  - `show_details`
- 默认值均为 `true`，确保首次启动行为与当前版本一致。

### 2) 本地缓存文件
- 新增 `RightPanelModuleVisibilityPath`（Resource）保存缓存文件路径。
- 缓存文件格式：JSON（`serde_json`），包含 `version` 与各布尔字段。
- 路径规则：
  1. 若设置 `AGENT_WORLD_VIEWER_MODULE_VISIBILITY_PATH`，优先使用。
  2. 否则使用 `$HOME/.agent_world_viewer/right_panel_modules.json`。
  3. 若 `HOME` 不可用，则回退到当前目录 `.agent_world_viewer/right_panel_modules.json`。

### 3) 调度
- 启动阶段：加载本地缓存并注入 `RightPanelModuleVisibilityState`。
- 运行阶段：当开关状态变更时自动写回本地文件。
- UI 阶段：右侧面板按开关状态决定各模块是否渲染。

## 里程碑
- **M1**：完成设计文档与项目管理文档。
- **M2**：实现模块开关状态资源、本地缓存读写与调度接线。
- **M3**：右侧面板增加模块开关 UI，并完成模块显隐联动。
- **M4**：补充测试、执行 `cargo check/test`、更新文档与 devlog。

## 风险
- **缓存文件损坏**：JSON 异常导致加载失败。
  - 缓解：读取失败时回退默认值并继续运行。
- **频繁落盘**：UI 频繁变更可能造成多次写文件。
  - 缓解：仅在状态变更时落盘（`Res::is_changed`）。
- **交互可恢复性**：若把所有模块都关掉，用户需仍可重新打开。
  - 缓解：模块开关区固定保留，不受各业务模块开关影响。
