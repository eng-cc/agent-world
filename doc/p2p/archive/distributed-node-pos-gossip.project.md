> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node PoS Gossip 协同（项目管理文档）

## 任务拆解
- [x] NPG-1：设计文档与项目管理文档落地。
- [x] NPG-2：在 `agent_world_node` 实现 UDP gossip endpoint 与 committed head 同步。
- [x] NPG-3：在 `world_viewer_live` 增加 gossip 参数接线与测试。
- [x] NPG-4：执行回归测试并收口文档/devlog。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_node/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/p2p/archive/distributed-node-pos-mainloop.md`

## 状态
- 当前阶段：NPG-4 完成，Node PoS gossip 协同已收口。
- 下一步：按需扩展为提案/投票消息 gossip 与跨节点 attestation 协同。
- 最近更新：2026-02-16。
