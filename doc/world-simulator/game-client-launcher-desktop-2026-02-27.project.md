# 可发行客户端启动器（Desktop）项目管理文档（2026-02-27）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [ ] T1 新增 `agent_world_client_launcher` 桌面 GUI 启动器（启动/停止/URL/日志）并补单元测试。
- [ ] T2 更新 `scripts/build-game-launcher-bundle.sh`，打包客户端启动器并生成 `run-client.sh`。
- [ ] T3 文档收口（项目状态、手册入口、验收记录）。

## 依赖
- 复用 `world_game_launcher` 作为核心编排执行器。
- 依赖 Web 预构建目录 `web/`（或自定义 `--viewer-static-dir`）。
- 桌面环境需具备图形会话以运行 GUI 启动器。

## 状态
- 当前阶段：进行中。
- 当前任务：T1（实现桌面 GUI 启动器）。
