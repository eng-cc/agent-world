# Viewer Live 完全事件驱动改造 Phase 10（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [ ] T1 代码收敛：`viewer/server` 删除 `tick_interval` 与定时回放推进
- [ ] T2 代码收敛：`viewer/web_bridge` 删除 `poll_interval` 与轮询 sleep
- [ ] T3 收口：活跃手册/入口示例与测试同步清理
- [ ] T4 回归与结项：required 测试 + 阶段文档收口

## 依赖
- `crates/agent_world/src/viewer/server.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `crates/agent_world/tests/viewer_offline_integration.rs`
- `site/index.html`
- `site/en/index.html`
- `testing-manual.md` / `doc/viewer-manual.md`（如需）

## 状态
- 当前阶段：进行中（T1）
