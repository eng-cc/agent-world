# 可发行统一启动器（Launcher）项目管理文档（2026-02-27）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 实现 `world_game_launcher`：参数解析、`world_viewer_live` 子进程托管、统一 URL 输出。
- [x] T2 为 launcher 接入内置静态 HTTP 服务（消费 prebuilt web dist）并补单元测试。
- [x] T3 提供发行打包脚本（生成 `bin/ + web/` 可分发目录）。
- [x] T4 文档收口与验收记录（手册入口/测试命令/状态更新）。

## 依赖
- doc/world-simulator/launcher/game-unified-launcher-2026-02-27.prd.md
- `world_viewer_live` 可执行文件（同 workspace 构建产物）。
- `crates/agent_world_viewer` 的 trunk 构建产物（web dist）。
- 运行平台需支持浏览器拉起命令（Linux/macOS/Windows）。

## 状态
- 当前阶段：已完成（T0~T4）。
- 当前任务：无（项目结项）。
- 审计备注（2026-03-04）：该专题为历史阶段记录，现行实现口径已切换到 `game-client-launcher-native-web-control-plane-unification-2026-03-04` 与 `game-client-launcher-web-console-2026-03-04`。
