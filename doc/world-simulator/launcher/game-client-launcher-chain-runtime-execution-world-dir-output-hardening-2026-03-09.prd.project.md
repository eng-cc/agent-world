# 启动器 chain runtime execution world 输出路径收敛（2026-03-09）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-chain-runtime-execution-world-dir-output-hardening-2026-03-09.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-033) [test_tier_required]: 完成“launcher execution world 输出路径收敛”PRD 建模与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-033) [test_tier_required]: 在 `world_game_launcher` / `world_web_launcher` 显式传递 `--execution-world-dir` 到 `output/chain-runtime/<node_id>/reward-runtime-execution-world`，并补齐定向回归测试。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
- `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
- `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`

## 状态
- 最近更新：2026-03-09
- 当前阶段: in_progress
- 当前任务: T1（代码实现 + 定向回归）
- 备注: 完成 T1 后需要同步回写模块总项目文档与当日日志。
