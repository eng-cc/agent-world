# Capture Viewer Frame（Agent UI截图闭环调试脚本）

## 目标
- 提供一个面向 agent 的一键闭环调试入口：`启动服务 -> 启动虚拟显示与 viewer -> 抓图 -> 留存日志`。
- 将可视化调试产物标准化到 `.tmp/screens/`，便于 agent 读取图片后继续定位问题。
- 默认在每次新调试开始前清空 `.tmp/`，避免历史残留影响判断。
- 增加平台识别与分支实现，使脚本在 Linux 与 macOS 都能完成最小截图闭环。
- 在 macOS 录屏权限受限时，使用 Bevy 内置截图能力完成窗口截图，避免依赖系统录屏授权。

## 范围
- **范围内**：
  - 新增脚本 `scripts/capture-viewer-frame.sh`。
  - 自动启动 `world_viewer_live`、`agent_world_viewer` 并抓取 `root.png`/`window.png`。
  - Linux 分支：使用 `Xvfb + xwininfo + ffmpeg` 完成无头抓图。
  - macOS 分支：优先使用 viewer 进程内截图（Bevy `Screenshot::primary_window`），不依赖 `screencapture` 权限。
  - viewer 新增可选“自动截图并退出”能力，通过环境变量控制输出路径与触发时机。
  - 输出统一日志与窗口几何信息（`live_server.log`/`viewer.log`/`window_geom.txt`）。
  - 默认清空 `.tmp/`，可通过 `--keep-tmp` 保留。
- **范围外**：
  - 不提供自动鼠标键盘交互回放。
  - 不替代完整 UI 自动化测试（仍以现有单测/联测为准）。

## 接口 / 数据
- 脚本路径：`scripts/capture-viewer-frame.sh`
- 典型调用：
  - `./scripts/capture-viewer-frame.sh`
  - `./scripts/capture-viewer-frame.sh --scenario llm_bootstrap --addr 127.0.0.1:5023 --tick-ms 300 --viewer-wait 8`
- 可选参数：
  - `--scenario` / `--addr` / `--tick-ms` / `--display` / `--width` / `--height` / `--viewer-wait` / `--llm` / `--keep-tmp`
  - `--auto-focus-target`：启动 viewer 后自动聚焦目标（如 `first_fragment`、`location:frag-1`、`agent:agent-0`）
  - `--auto-focus-radius`：自动聚焦半径覆盖值
  - `--auto-focus-keep-2d`：自动聚焦时保持 2D（默认切换 3D）
- viewer 内置截图环境变量：
  - `AGENT_WORLD_VIEWER_CAPTURE_PATH`：截图输出文件路径（PNG）。
  - `AGENT_WORLD_VIEWER_CAPTURE_DELAY_SECS`：最短等待秒数（默认 2 秒）。
  - `AGENT_WORLD_VIEWER_CAPTURE_MAX_WAIT_SECS`：无快照时的最大等待秒数（默认 15 秒）。
  - （可选）`AGENT_WORLD_VIEWER_AUTO_FOCUS*`：脚本在传入 `--auto-focus-*` 时自动注入。
- 输出目录：`.tmp/screens/`
  - `root.png`：整屏截图（macOS 内置截图模式下与 `window.png` 相同）
  - `window.png`：viewer 窗口截图
  - `live_server.log` / `viewer.log` / `xvfb.log`
  - `window_line.txt` / `window_geom.txt`

## 里程碑
- **M1**：输出脚本设计文档与项目管理文档。
- **M2**：实现脚本与参数解析，接入清空 `.tmp/` 机制。
- **M3**：更新 AGENTS/README/任务日志并完成运行验证。
- **M4**：补充 Linux/macOS 平台分支逻辑与依赖检查。
- **M5**：接入 viewer 内置自动截图并在 macOS 默认启用，绕过录屏权限约束。

## 风险
- **依赖差异**：Linux 与 macOS 命令能力不同，需要分别校验命令可用性。
- **截图时机**：若触发过早可能抓到“连接中”界面，需配合 `--viewer-wait` 与内置延迟参数。
- **渲染链路**：viewer 内置截图依赖 Bevy 渲染完成回调，若渲染异常可能导致截图未落盘。
- **资源占用**：脚本会启动 viewer/server，调试完成后必须清理后台进程（脚本已 trap 清理）。
