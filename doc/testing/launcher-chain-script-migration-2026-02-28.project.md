# 启动链路脚本迁移（2026-02-28）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 迁移日常脚本：`run-game-test.sh`、`viewer-release-qa-loop.sh` 改为 `world_game_launcher`。
- [ ] T2 长跑脚本阻断：`s10-five-node-game-soak.sh`、`p2p-longrun-soak.sh` 启动前显式失败并提示迁移方向。
- [ ] T3 文档收口：更新手册口径与项目状态，补任务日志。

## 依赖
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- 现有测试脚本与 `testing-manual.md`

## 状态
- 当前阶段：进行中（T0~T1 已完成，执行 T2）。
- 当前任务：T2 长跑脚本阻断。
