# Viewer WebSocket/HTTP Bridge（项目管理文档）

## 任务拆解

### WLB1 文档建模
- [x] WLB1.1 输出设计文档（`doc/world-simulator/viewer-websocket-http-bridge.md`）
- [x] WLB1.2 输出项目管理文档（本文件）
- [x] WLB1.3 在总项目文档挂载任务入口

### WLB2 后端 bridge
- [x] WLB2.1 实现 WebSocket <-> TCP line protocol 双向桥接
- [x] WLB2.2 `world_viewer_live` 增加 `--web-bind` 参数并接入 bridge 生命周期
- [x] WLB2.3 补充测试并通过 `test_tier_required` 最小回归

### WLB3 Web Viewer 接入
- [ ] WLB3.1 wasm 路径接入 WebSocket 客户端（替代固定 offline）
- [ ] WLB3.2 支持 WebSocket 地址配置（默认 + URL 参数）
- [ ] WLB3.3 通过 wasm 编译回归与 viewer 相关最小测试

### WLB4 文档与闭环收口
- [ ] WLB4.1 更新 AGENTS/手册/运行路径文档（含 llm_bootstrap Web 命令）
- [ ] WLB4.2 执行 Web 端闭环验证（live server + web viewer + Playwright）
- [ ] WLB4.3 更新项目状态、开发日志并收口

## 依赖
- `crates/agent_world/src/viewer/live.rs`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `crates/agent_world_viewer/src/main.rs`
- `doc/viewer-manual.md`
- `AGENTS.md`
- `doc/world-simulator/viewer-bevy-web-runtime.md`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：WLB1~WLB2 已完成，WLB3~WLB4 进行中。
- 下一步：接入 wasm WebSocket 客户端，打通 Web 在线路径。
- 最近更新：2026-02-15（完成后端 bridge 与 `--web-bind` 接入，回归通过）。
