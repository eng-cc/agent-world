# Viewer 启动自动选中（用于截图闭环）

## 目标
- 在 viewer 启动后支持“自动选中指定对象”，用于截图闭环中稳定产出“右侧选中详情”画面。
- 支持与现有自动聚焦能力配合，形成 `自动聚焦 + 自动选中 + 自动截图` 的无人工链路。
- 不改变默认交互路径；仅在显式配置环境变量时启用。

## 范围
### 范围内
- `agent_world_viewer` 新增自动选中配置与系统（按目标自动填充 `ViewerSelection`）。
- 支持目标类型：`agent:<id>`、`location:<id>`、`first_agent`、`first_location`。
- `scripts/capture-viewer-frame.sh` 新增参数 `--auto-select-target`，并透传 viewer 环境变量。
- 新增/更新单元测试，覆盖配置解析与自动选中行为。

### 范围外
- 不改鼠标点击拾取逻辑。
- 不改右侧详情字段内容（仅保证截图链路可稳定触发已有字段）。
- 不实现资产/设施/chunk 的自动选中别名（后续按需补充）。

## 接口 / 数据
- viewer 环境变量：
  - `AGENT_WORLD_VIEWER_AUTO_SELECT=1`
  - `AGENT_WORLD_VIEWER_AUTO_SELECT_TARGET=<target>`
- target 语法：
  - `agent:agent-0`
  - `location:loc-1`
  - `first_agent`
  - `first_location`
- 脚本参数：
  - `--auto-select-target <target>`：自动注入上述环境变量。

## 里程碑
- **ASC-1**：设计文档与项目文档。
- **ASC-2**：viewer 自动选中系统与测试。
- **ASC-3**：截图脚本参数透传与闭环验证。

## 风险
- 事件驱动重建会清空选择，需确保自动选中能在后续帧恢复，避免截图窗口错过详情。
- 若 target 无效或对象未生成，自动选中应静默降级，不影响 viewer 启动。
- 自动选中开启时会覆盖手工选择，不应默认启用。
