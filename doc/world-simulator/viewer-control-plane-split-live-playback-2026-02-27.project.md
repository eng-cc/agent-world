# Viewer 控制面拆分：回放/Live 分离（2026-02-27）项目管理

## 任务拆解
- [x] T0 建立设计文档与项目管理文档
- [x] T1 协议层与 server/live 控制处理拆分（含兼容桥接）
- [ ] T2 viewer 控制发送按 profile 路由并收敛 seek 发送
- [ ] T3 测试、文档收口与 devlog 收口

## 依赖
- `doc/world-simulator/viewer-control-plane-split-live-playback-2026-02-27.md`
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world/src/viewer/{protocol.rs,server.rs,live_split_part2.rs,mod.rs}`
- `crates/agent_world_viewer/src/{main_connection.rs,timeline_controls.rs,egui_right_panel_controls.rs,web_test_api.rs,headless.rs}`
- `doc/devlog/2026-02-27.md`

## 状态
- 当前阶段：进行中（下一步 T2）
- 备注：与“live 禁用 seek（P2P 不可回退）”策略保持一致。
