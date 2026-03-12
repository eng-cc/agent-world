# 启动器 chain runtime stale execution world 恢复（2026-03-12）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-chain-runtime-stale-execution-world-recovery-2026-03-12.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-chain-runtime-stale-execution-world-recovery-2026-03-12.prd.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-034) [test_tier_required]: 完成“launcher stale execution world 恢复”PRD/Design/Project 建模与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-034) [test_tier_required]: 为 launcher / GUI Agent 增加 stale execution world 结构化错误识别，并在状态层暴露恢复建议。
- [x] T2 (PRD-WORLD_SIMULATOR-034) [test_tier_required]: 实现 fresh node id 恢复链路与回归验证，覆盖 `start_chain -> query_explorer_overview -> start_game` 最小闭环。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
- `crates/agent_world/src/bin/world_web_launcher/gui_agent_api.rs`
- `crates/agent_world_client_launcher/src/*`

## 状态
- 最近更新：2026-03-12
- 当前阶段: completed
- 当前任务: `none`
- owner: `viewer_engineer`
- 联审: `runtime_engineer`
- 备注: `T0/T1/T2` 已完成；`world_web_launcher` 已新增 stale execution world 结构化状态、GUI Agent `recover_chain` 动作与 fresh node id 恢复建议，`agent_world_client_launcher` 已补齐一键恢复 CTA 与状态映射。
