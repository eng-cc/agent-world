# 可发行客户端启动器（Desktop）项目管理文档（2026-02-27）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 新增 `agent_world_client_launcher` 桌面 GUI 启动器（启动/停止/URL/日志）并补单元测试。
- [x] T2 更新 `scripts/build-game-launcher-bundle.sh`，打包客户端启动器并生成 `run-client.sh`。
- [x] T3 文档收口（项目状态、手册入口、验收记录）。
- [x] T4 修复客户端启动器中文乱码：接入 CJK 字体 fallback（支持环境变量覆盖）。

## 依赖
- 复用 `world_game_launcher` 作为核心编排执行器。
- 依赖 Web 预构建目录 `web/`（或自定义 `--viewer-static-dir`）。
- 桌面环境需具备图形会话以运行 GUI 启动器。

## 状态
- 当前阶段：已完成（T0~T4）。
- 当前任务：无（已结项）。
