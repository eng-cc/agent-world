# Agent World Runtime：生产级区块链 + P2P FS 路线图（项目管理文档）

## 任务拆解
- [x] PRG-1：完成路线图设计文档与项目管理文档。
- [x] PRG-2：实现 `agent_world_node` 链式 `block_hash`（含状态持久化兼容）。
- [x] PRG-3：实现奖励结算 `RewardSettlementEnvelope` 传输签名与消费端验签。
- [ ] PRG-4：补齐测试并执行 `test_tier_required` 回归，回写文档与 devlog。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/pos_state_store.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live/reward_runtime_network.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：PRG-1 ~ PRG-3 完成，PRG-4 进行中。
- 阻塞项：无。
- 最近更新：2026-02-17。
