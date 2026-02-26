# viewer-web-playability-unblock-2026-02-26 项目管理

## 任务拆解
- [x] T0 建立设计文档与项目管理文档
- [x] T1 修复 `web_test_api` 的 `runSteps`/`sendControl` 入参契约，消除类型不匹配 panic。
- [ ] T2 增加 wasm + Player 模式自动 `Play`，确保连接后默认可推进。
- [ ] T3 修复 `scripts/run-game-test.sh` 的 WS 就绪探针，消除 `HandshakeIncomplete` 假故障。
- [ ] T4 运行回归测试并回写文档/日志。

## 依赖
- `doc/world-simulator/viewer-web-playability-unblock-2026-02-26.md`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/headless.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `scripts/run-game-test.sh`

## 状态
- 当前阶段：进行中（T2）
- 最近更新：2026-02-26
