# Agent World Runtime：Node PoS 主循环接入（项目管理文档）

## 任务拆解
- [x] NPOS-1：设计文档与项目管理文档落地。
- [x] NPOS-2：重构 `crates/node` 为 PoS 驱动主循环，并将包名迁移为 `agent_world_node`。
- [ ] NPOS-3：在 `world_viewer_live` 启动流程接线 `agent_world_node` 并更新测试。
- [ ] NPOS-4：执行回归测试，回写文档状态与 devlog 收口。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/node/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/pos.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/world-runtime/distributed-pos-consensus.md`

## 状态
- 当前阶段：NPOS-2 完成，进入 NPOS-3。
- 下一步：完善 `world_viewer_live` 启动参数到 PoS 配置接线并补测试。
- 最近更新：2026-02-16。
