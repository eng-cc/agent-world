# Agent World Runtime：共识代码统一收敛到 agent_world_consensus（项目管理文档）

## 任务拆解
- [x] CCG-0：完成设计文档与项目管理文档。
- [x] CCG-1：在 `agent_world_consensus` 实现可复用 node_pos 核心状态机，并在 `agent_world_node` 完成接线替换。
- [x] CCG-2：补齐/回归测试并回写文档与 devlog。
- [x] CCG-3：扩展设计/项目文档，定义第二阶段全收口任务。
- [x] CCG-4：将 `agent_world_node` 残留共识纯逻辑（action/signature/message）迁移到 `agent_world_consensus` 并接线。
- [x] CCG-5：完成 PoS 单链路收敛（`pos` 复用 `node_pos` 推进核心）、定向回归和文档/devlog 收口。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/p2p/consensus-code-consolidation-to-agent-world-consensus.md`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/node_consensus_action.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/node_consensus_message.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/node_consensus_signature.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/dht.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/network.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/node_pos.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/pos.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/gossip_udp.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/node_runtime_core.rs`

## 状态
- 当前阶段：CCG-0~CCG-5 全部完成。
- 阻塞项：无。
- 最近更新：2026-02-20。
