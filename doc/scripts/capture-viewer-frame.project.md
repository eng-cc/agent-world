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

## 依赖
- `Xvfb`
- `ffmpeg`
- `xwininfo`
- Rust/Cargo（`world_viewer_live` + `agent_world_viewer`）

## 状态
- 当前阶段：M3（实现与文档同步完成）
- 下一阶段：按需补充“多帧抓图”或“自动交互注入”
- 最近更新：新增一键 UI 截图闭环调试脚本并默认清空 `.tmp/`（2026-02-07）
