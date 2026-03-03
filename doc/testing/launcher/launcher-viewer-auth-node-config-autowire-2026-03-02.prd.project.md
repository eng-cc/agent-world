# Agent World: 启动器 Viewer 鉴权自动继承 Node 配置（2026-03-02）（项目管理）

## 任务拆解（含 PRD-ID 映射）
- [x] AUTOWIRE-1 (PRD-TESTING-LAUNCHER-AUTH-001): 完成专题设计文档与项目管理文档建档。
- [x] AUTOWIRE-2 (PRD-TESTING-LAUNCHER-AUTH-001/002): `world_game_launcher` 在 Web `index.html` 注入 Viewer 鉴权配置并补测试。
- [x] AUTOWIRE-3 (PRD-TESTING-LAUNCHER-AUTH-002/003): `agent_world_viewer` 增加 wasm 注入读取与 native `config.toml[node]` 回退并补测试。
- [x] AUTOWIRE-4 (PRD-TESTING-LAUNCHER-AUTH-003): 完成定向回归、文档状态与 devlog 收口。
- [x] AUTOWIRE-5 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.prd.project.md`。

## 依赖
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat_auth.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat_tests.rs`
- `crates/agent_world_viewer/Cargo.toml`
- `config.toml` (`[node] private_key/public_key`)
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`
- `doc/devlog/2026-03-02.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
