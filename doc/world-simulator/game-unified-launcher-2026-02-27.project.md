# 可发行统一启动器（Launcher）项目管理文档（2026-02-27）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 实现 `world_game_launcher`：参数解析、`world_viewer_live` 子进程托管、统一 URL 输出。
- [x] T2 为 launcher 接入内置静态 HTTP 服务（消费 prebuilt web dist）并补单元测试。
- [x] T3 提供发行打包脚本（生成 `bin/ + web/` 可分发目录）。
- [ ] T4 文档收口与验收记录（手册入口/测试命令/状态更新）。

## 依赖
- `world_viewer_live` 可执行文件（同 workspace 构建产物）。
- `crates/agent_world_viewer` 的 trunk 构建产物（web dist）。
- 运行平台需支持浏览器拉起命令（Linux/macOS/Windows）。

## 状态
- 当前阶段：进行中。
- 当前任务：T4（文档收口与验收记录）。
