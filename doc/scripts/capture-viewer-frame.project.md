# Capture Viewer Frame（Agent UI截图闭环调试脚本）（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/scripts/capture-viewer-frame.md`）
- [x] 输出项目管理文档（本文件）
- [x] 新增调试脚本（`scripts/capture-viewer-frame.sh`）
- [x] 默认每次调试前清空 `.tmp/`（支持 `--keep-tmp`）
- [x] 更新 `.gitignore` 忽略 `.tmp/`
- [x] 更新 `AGENTS.md`（优先脚本说明）
- [x] 更新 `README.md`（脚本入口）
- [x] 运行验证（`./scripts/capture-viewer-frame.sh --help` + 实际抓图）
- [x] 更新任务日志（`doc/devlog/2026-02-07.md`）
- [x] M4：按平台分支截图实现（Linux: Xvfb/ffmpeg，macOS: screencapture/osascript）
- [x] M4：脚本自动识别 `uname` 并切换依赖检查与抓图流程
- [x] M4：保持输出产物路径与文件命名一致（`root.png` / `window.png` / `window_geom.txt`）
- [x] M4：补充文档与任务日志并验证脚本语法/帮助信息
- [x] M5：viewer 新增内置自动截图能力（Bevy screenshot API）
- [x] M5：脚本 macOS 分支切换为“进程内截图”链路，规避系统录屏权限
- [x] M5：新增/更新测试（内置截图配置解析与触发条件）
- [x] M5：验证脚本在 macOS 下可产出有效 `window.png` 并可用于 UI 评审

## 依赖
- Linux：`Xvfb`、`ffmpeg`、`xwininfo`
- macOS：无需系统录屏权限（使用 viewer 进程内截图）
- Rust/Cargo（`world_viewer_live` + `agent_world_viewer`）

## 状态
- 当前阶段：M5（内置截图链路完成）
- 下一阶段：按需补充“多帧抓图”或“自动交互注入”
- 最近更新：macOS 默认改用 Bevy 内置截图，避免录屏权限失败（2026-02-07）
