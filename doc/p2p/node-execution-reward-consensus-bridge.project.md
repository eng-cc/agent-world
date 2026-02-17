# Agent World Runtime：节点执行桥接与奖励共识触发闭环（项目管理文档）

## 任务拆解
- [x] ERCB-0：完成设计文档与项目管理文档。
- [x] ERCB-1：实现执行桥接器（`committed_height -> RuntimeWorld` 执行记录 + CAS 落盘 + 状态恢复）。
- [x] ERCB-2：实现奖励结算网络触发（结算包发布/订阅/应用，含单机 fallback）。
- [ ] ERCB-3：实现多观察者签名轨迹（发布、验签、去重、collector 接线、报表输出）。
- [ ] ERCB-4：补齐测试与 CLI 参数覆盖，执行 `test_tier_required` 回归。
- [ ] ERCB-5：回写项目状态与 devlog 收口。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/p2p/node-execution-reward-consensus-bridge.md`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live/reward_runtime_settlement.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live/distfs_probe_runtime.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/node_points_runtime.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：ERCB-0~ERCB-2 已完成，进入 ERCB-3。
- 阻塞项：无。
- 最近更新：2026-02-17。
