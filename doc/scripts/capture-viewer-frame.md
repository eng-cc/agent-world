# Capture Viewer Frame（Agent UI截图闭环调试脚本）

## 目标
- 提供一个面向 agent 的一键闭环调试入口：`启动服务 -> 启动虚拟显示与 viewer -> 抓图 -> 留存日志`。
- 将可视化调试产物标准化到 `.tmp/screens/`，便于 agent 读取图片后继续定位问题。
- 默认在每次新调试开始前清空 `.tmp/`，避免历史残留影响判断。
- 增加平台识别与分支实现，使脚本在 Linux 与 macOS 都能完成最小截图闭环。

## 范围
- **范围内**：
  - 新增脚本 `scripts/capture-viewer-frame.sh`。
  - 自动启动 `world_viewer_live`、`agent_world_viewer` 并抓取 `root.png`/`window.png`。
  - Linux 分支：使用 `Xvfb + xwininfo + ffmpeg` 完成无头抓图。
  - macOS 分支：使用 `osascript + screencapture` 完成本机窗口抓图。
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
- 输出目录：`.tmp/screens/`
  - `root.png`：整屏截图
  - `window.png`：viewer 窗口截图
  - `live_server.log` / `viewer.log` / `xvfb.log`
  - `window_line.txt` / `window_geom.txt`

## 里程碑
- **M1**：输出脚本设计文档与项目管理文档。
- **M2**：实现脚本与参数解析，接入清空 `.tmp/` 机制。
- **M3**：更新 AGENTS/README/任务日志并完成运行验证。
- **M4**：补充 Linux/macOS 平台分支逻辑与依赖检查。

## 风险
- **依赖差异**：Linux 与 macOS 命令能力不同，需要分别校验命令可用性。
- **窗口识别**：macOS 依赖 `System Events` 读取窗口信息，可能受系统权限（辅助功能权限）影响。
- **首帧时机**：若等待时间不足可能抓到黑屏或未完整渲染画面，需要增大 `--viewer-wait`。
- **资源占用**：脚本会启动 viewer/server，调试完成后必须清理后台进程（脚本已 trap 清理）。
