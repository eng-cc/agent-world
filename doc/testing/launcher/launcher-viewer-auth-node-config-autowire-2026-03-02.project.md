# 启动器 Viewer 鉴权自动继承 Node 配置（2026-03-02）项目管理

## 任务拆解
- [x] T0 (PRD-TESTING-001)：建档（新增设计文档与项目管理文档）。
- [x] T1 (PRD-TESTING-002)：启动器注入（`world_game_launcher` 在 Web `index.html` 注入 Viewer 鉴权配置，来自 `config.toml [node]`，补单测）。
- [x] T2 (PRD-TESTING-002)：Viewer 回退（`agent_world_viewer` 增加 wasm 注入读取与 native `config.toml` 回退，补单测）。
- [x] T3 (PRD-TESTING-003)：回归与收口（执行定向测试、更新项目状态与 devlog）。

## 依赖
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat_auth.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat_tests.rs`
- `crates/agent_world_viewer/Cargo.toml`
- `doc/devlog/2026-03-02.md`

## 状态
- 当前阶段：已完成
- 当前任务：无
- 进度：T0/T1/T2/T3 全部完成
