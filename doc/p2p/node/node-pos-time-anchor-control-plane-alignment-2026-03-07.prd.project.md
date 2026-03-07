# Agent World Runtime：PoS 时间锚定控制面参数与可观测口径对齐（项目管理文档）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-P2P-010-T0 (PRD-P2P-NODE-SURFACE-001/002/003) [test_tier_required]: 完成专题 PRD 与项目管理建档，并回写 `doc/p2p/prd.md` / `doc/p2p/prd.project.md` / `doc/p2p/prd.index.md`。
- [ ] TASK-P2P-010-T1 (PRD-P2P-NODE-SURFACE-001) [test_tier_required]: `world_chain_runtime/world_viewer_live` 新增 PoS 时间锚定参数解析与 `NodePosConfig` 映射，并明确 `node_tick_ms` 轮询语义。
- [ ] TASK-P2P-010-T2 (PRD-P2P-NODE-SURFACE-002) [test_tier_required]: launcher UI/配置字段扩展与参数透传对齐，补齐校验提示。
- [ ] TASK-P2P-010-T3 (PRD-P2P-NODE-SURFACE-003) [test_tier_required]: 更新 longrun/s10 脚本、release 示例与相关文档口径；保持旧参数兼容。
- [ ] TASK-P2P-010-T4 (PRD-P2P-NODE-SURFACE-001/002/003) [test_tier_required + test_tier_full]: 补齐定向测试与闭环回归，收口模块项目状态。

## 依赖
- `doc/p2p/node/node-pos-time-anchor-control-plane-alignment-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`
- `crates/agent_world_client_launcher/src/launcher_core.rs`
- `crates/agent_world_client_launcher/src/llm_settings.rs`
- `crates/agent_world_client_launcher/src/llm_settings_web.rs`
- `crates/agent_world_launcher_ui/src/lib.rs`
- `scripts/p2p-longrun-soak.sh`
- `scripts/s10-five-node-game-soak.sh`
- `world_viewer_live.release.example.toml`

## 状态
- 更新日期: 2026-03-07
- 当前状态: in_progress
- 下一任务: `TASK-P2P-010-T1`
- 阻塞项: 无
- 进展: `TASK-P2P-010-T0` 已完成，已完成专题文档建档与模块映射回写。
- 说明: 本文档仅维护执行计划与任务状态；实施过程记录写入 `doc/devlog/2026-03-07.md`。
