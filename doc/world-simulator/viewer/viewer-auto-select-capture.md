# Viewer 启动自动化步骤（相机/选中，用于截图闭环）

## 目标
- 在 viewer 启动后支持“自动执行步骤”，覆盖相机模式切换、聚焦、平移、缩放、轨道旋转与对象选中。
- 支持与现有自动截图能力配合，形成 `自动化步骤 + 自动截图` 的无人工链路，可复用于任意功能验证。
- 不改变默认交互路径；仅在显式配置环境变量时启用。

## 范围
### 范围内
- `agent_world_viewer` 新增启动自动化系统（按步骤驱动相机与选中状态）。
- 支持步骤类型：`wait`、`mode`、`focus`、`pan`、`zoom`、`orbit`、`select`。
- 支持目标类型：`agent:<id>`、`location:<id>`、`first_agent`、`first_location`。
- 保留向后兼容：`AGENT_WORLD_VIEWER_AUTO_SELECT*` 仍可单独触发自动选中。
- `scripts/capture-viewer-frame.sh` 新增参数 `--auto-select-target` 与 `--automation-steps`，并透传 viewer 环境变量。
- 新增/更新测试，覆盖配置解析与相机/选中步骤语法。

### 范围外
- 不改鼠标点击拾取逻辑。
- 不改右侧详情字段内容（仅保证截图链路可稳定触发已有字段）。
- 不实现资产/设施/chunk 的自动选中别名（后续按需补充）。

## 接口 / 数据
- viewer 环境变量：
  - `AGENT_WORLD_VIEWER_AUTO_SELECT=1`
  - `AGENT_WORLD_VIEWER_AUTO_SELECT_TARGET=<target>`
  - `AGENT_WORLD_VIEWER_AUTOMATION_STEPS=<steps>`
- target 语法：
  - `agent:agent-0`
  - `location:loc-1`
  - `first_agent`
  - `first_location`
- steps 语法（`;` 分隔）：
  - `mode=3d|2d`
  - `focus=<target>`
  - `pan=<x,y,z>`
  - `zoom=<factor>`
  - `orbit=<yaw_deg,pitch_deg>`
  - `select=<target>`
  - `wait=<seconds>`
- 脚本参数：
  - `--auto-select-target <target>`：自动注入上述环境变量。
  - `--automation-steps "<steps>"`：启动后自动执行步骤序列。

## 里程碑
- **ASC-1**：设计文档与项目文档。
- **ASC-2**：viewer 自动化配置解析（兼容 auto-select）。
- **ASC-3**：viewer 自动化执行系统（相机 + 选中）。
- **ASC-4**：自动化解析/默认模式测试补齐。
- **ASC-5**：截图脚本参数透传（auto-select + automation-steps）。
- **ASC-6**：闭环验证与任务日志更新。

## 风险
- 事件驱动重建会清空选择，需确保 `select` 步骤可在对象可用后重试，避免错过截图窗口。
- 若 target 无效或对象未生成，自动化应静默等待/降级，不影响 viewer 启动。
- 自动化开启时会覆盖手工相机/选择状态，不应默认启用。
