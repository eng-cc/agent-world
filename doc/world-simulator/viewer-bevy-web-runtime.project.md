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
- [x] WBR4.3 Playwright 闭环验证（打开 Web 页面、抓 snapshot/screenshot、校验 console error）

### WBR5 闭环策略对齐
- [x] WBR5.1 与 `viewer-web-closure-testing-policy` 对齐，明确 Web 为默认闭环路径
- [x] WBR5.2 在手册/AGENTS/脚本文档中统一“Web 默认，native fallback”口径

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/index.html`
- `scripts/run-viewer-web.sh`
- `doc/viewer-manual.md`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：WBR1~WBR5 全部完成。
- 下一步：与 `viewer-websocket-http-bridge` 持续对齐在线链路回归口径。
- 最近更新：2026-02-15（在线 bridge 已接入，Web 默认闭环保持生效）。
