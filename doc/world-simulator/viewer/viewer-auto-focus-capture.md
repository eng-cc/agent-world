# Agent World Simulator：Viewer 自动聚焦（人机共用 + 截图闭环）设计文档

## 目标
- 为人工调试提供“一键聚焦”能力：在对象密集场景中快速把相机对准目标，减少手动拖拽成本。
- 为截图闭环提供“可脚本化聚焦”能力：在 `capture-viewer-frame.sh` 中可传参触发自动聚焦，保证截图可重复、可对齐。
- 与现有渲染/协议兼容：不改 `agent_world` 协议，仅在 viewer 本地消费现有 snapshot/scene 数据。

## 范围

### In Scope
- `agent_world_viewer` 新增自动聚焦配置（环境变量）与一次性启动聚焦流程。
- `agent_world_viewer` 新增人工快捷键（`F`）聚焦当前选中对象。
- `scripts/capture-viewer-frame.sh` 新增自动聚焦参数并映射到 viewer 环境变量。
- 补充单元测试（配置解析/目标选择）与文档。

### Out of Scope
- 不改动 world server 或 simulator 事件协议。
- 不新增复杂路径镜头、平滑动画时间轴、录像导出。
- 不强制改变默认观察行为（无参数时保持当前相机逻辑）。

## 接口 / 数据

### Viewer 环境变量
- `AGENT_WORLD_VIEWER_AUTO_FOCUS`
  - `true/1/on` 启用启动自动聚焦。
- `AGENT_WORLD_VIEWER_AUTO_FOCUS_TARGET`
  - 目标选择器，支持：
    - `first_fragment`
    - `first_location`
    - `first_agent`
    - `location:<id>`
    - `agent:<id>`
- `AGENT_WORLD_VIEWER_AUTO_FOCUS_FORCE_3D`
  - 是否在自动聚焦时切到 3D（默认 `true`）。
- `AGENT_WORLD_VIEWER_AUTO_FOCUS_RADIUS`
  - 可选，覆盖聚焦半径（米制 world unit）。

### 人工交互
- 快捷键 `F`：对当前选中实体执行相机聚焦（不改变选中逻辑）。

### 脚本参数
- `--auto-focus-target <target>`：透传到 `AGENT_WORLD_VIEWER_AUTO_FOCUS_TARGET`。
- `--auto-focus-radius <value>`：透传到 `AGENT_WORLD_VIEWER_AUTO_FOCUS_RADIUS`。
- `--auto-focus-keep-2d`：将 `AGENT_WORLD_VIEWER_AUTO_FOCUS_FORCE_3D=0`。

## 里程碑
- **AFC1**：文档与项目任务拆解。
- **AFC2**：viewer 启动自动聚焦 + 手动快捷键聚焦。
- **AFC3**：截图脚本参数接入与闭环验证。

## 风险
- 聚焦目标不存在（ID 错误或场景尚未生成）时需安全降级为不动作。
- 自动切 3D 若与人工操作冲突，需限定为“仅启动一次”。
- 截图时间窗口过短会在聚焦前截图，需要脚本等待参数兜底。
