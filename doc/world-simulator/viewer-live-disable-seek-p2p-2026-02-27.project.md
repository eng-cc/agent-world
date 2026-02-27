# Viewer Live 禁用 Seek（P2P 不可回退）2026-02-27 项目管理

## 任务拆解
- [x] T0 建立设计文档与项目管理文档
- [x] T1 改造 live 控制处理：禁用 `ViewerControl::Seek`
- [ ] T2 收敛 viewer/web_test_api 玩家与测试入口动作集合（移除 seek 暴露）
- [ ] T3 更新测试与文档收口（含 devlog）

## 依赖
- `doc/world-simulator/viewer-live-disable-seek-p2p-2026-02-27.md`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/egui_right_panel_controls.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`

## 状态
- 当前阶段：进行中（下一步 T2）
- 备注：遵循“P2P live 单调前进、不可 seek 回退”的约束。
