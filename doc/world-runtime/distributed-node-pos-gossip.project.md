# Agent World Runtime：Node PoS Gossip 协同（项目管理文档）

## 任务拆解
- [x] NPG-1：设计文档与项目管理文档落地。
- [ ] NPG-2：在 `agent_world_node` 实现 UDP gossip endpoint 与 committed head 同步。
- [ ] NPG-3：在 `world_viewer_live` 增加 gossip 参数接线与测试。
- [ ] NPG-4：执行回归测试并收口文档/devlog。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/node/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/world-runtime/distributed-node-pos-mainloop.md`

## 状态
- 当前阶段：NPG-1 完成，进入 NPG-2。
- 下一步：实现节点 gossip endpoint 与网络 head 汇总状态。
- 最近更新：2026-02-16。
