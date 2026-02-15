# Viewer Bevy 浏览器运行路径（项目管理文档）

## 任务拆解

### WBR1 文档建模
- [x] WBR1.1 输出设计文档（`doc/world-simulator/viewer-bevy-web-runtime.md`）
- [x] WBR1.2 输出项目管理文档（本文件）
- [x] WBR1.3 在总项目文档挂载任务入口

### WBR2 wasm 兼容改造
- [x] WBR2.1 修复 `agent_world` 的 wasm 编译不兼容（LLM HTTP client）
- [x] WBR2.2 修复 `agent_world_viewer` 的 wasm 编译不兼容（Web 离线路径）
- [x] WBR2.3 执行 `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown`

### WBR3 Web 入口与手册
- [x] WBR3.1 新增 `trunk` 页面入口（`crates/agent_world_viewer/index.html`）
- [x] WBR3.2 新增 Web 启动脚本（`scripts/run-viewer-web.sh`）
- [x] WBR3.3 更新 `doc/viewer-manual.md` Web 运行说明

### WBR4 回归与收口
- [x] WBR4.1 执行 `test_tier_required` 最小回归（至少 viewer 相关 check/test）
- [x] WBR4.2 更新本项目文档状态、总项目文档与开发日志

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/index.html`
- `scripts/run-viewer-web.sh`
- `doc/viewer-manual.md`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：WBR1~WBR4 全部完成。
- 下一步：若需浏览器在线模式，新增 WebSocket/HTTP bridge 路径对接 `world_viewer_live`。
- 最近更新：2026-02-15（完成 wasm 兼容、Web 入口与回归收口）。
