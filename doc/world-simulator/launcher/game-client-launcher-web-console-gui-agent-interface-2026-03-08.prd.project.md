# 客户端启动器 Web Console GUI Agent 全量接口（2026-03-08）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-031) [test_tier_required]: 完成 GUI Agent 接口专题 PRD 建模、验收冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-031) [test_tier_required]: 在 `world_web_launcher` 落地 `/api/gui-agent/capabilities|state|action`，覆盖人工操作全功能动作映射。
- [ ] T2 (PRD-WORLD_SIMULATOR-031) [test_tier_required]: 补齐 `world_web_launcher` GUI Agent 接口定向测试并完成模块 PRD/project/devlog 收口。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
- `crates/agent_world/src/bin/world_web_launcher/transfer_query_proxy.rs`
- `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-08
- 当前阶段: in_progress
- 当前任务: T1
- 备注: 目标是让 GUI Agent 通过 Web Console 机器接口覆盖现有人工操作能力。
