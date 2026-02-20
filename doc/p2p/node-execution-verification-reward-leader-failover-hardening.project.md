# Agent World Runtime：节点执行校验与奖励 Leader/Failover 生产化收口（项目管理文档）

## 任务拆解
- [x] T0：完成设计文档与项目管理文档。
- [x] T1：实现 `agent_world_node` 执行校验强化（配置、入站校验、gap-sync 执行一致性校验、快照可观测字段）。
- [x] T2：实现 `world_viewer_live` reward runtime leader/failover 策略与生产默认接线。
- [x] T3：补齐测试并执行定向回归（node + world_viewer_live），回写文档与 devlog。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/p2p/node-execution-verification-reward-leader-failover-hardening.md`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/types.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`

## 状态
- 当前阶段：T0~T3 已全部完成。
- 阻塞项：无。
- 最近更新：2026-02-20。
