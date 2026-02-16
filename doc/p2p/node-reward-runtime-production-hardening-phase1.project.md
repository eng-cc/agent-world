# Agent World Runtime：节点奖励运行时生产化加固（Phase 1，项目管理文档）

## 任务拆解
- [x] PRH1-1：完成设计文档与项目管理文档。
- [x] PRH1-2：实现 `NodePointsLedger/Collector` 快照导出与恢复，并接入 reward runtime 状态文件。
- [ ] PRH1-3：实现兑换签名授权策略 `require_redeem_signer_match_node_id` 并接入默认生产配置。
- [ ] PRH1-4：移除 reward runtime 占位身份绑定，改为显式绑定与错误收口。
- [ ] PRH1-5：补齐/更新测试，执行 `test_tier_required` 回归，回写文档状态与 devlog。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/node_points.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/node_points_runtime.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/reward_asset.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/world/event_processing.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：PRH1-1 ~ PRH1-2 已完成，PRH1-3 ~ PRH1-5 进行中。
- 阻塞项：无。
- 最近更新：2026-02-17。
