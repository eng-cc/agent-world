# Agent World Runtime：Node 主循环基础模块（项目管理文档）

## 任务拆解
- [x] DNM-1：设计文档与项目管理文档落地。
- [x] DNM-2：新增 `crates/node`，实现 `NodeRole/NodeConfig/NodeRuntime/NodeSnapshot` 与单元测试。
- [x] DNM-3：在 `world_viewer_live` 启动流程中接入节点自动启动，并补充 CLI 参数解析测试。
- [ ] DNM-4：执行回归测试、回写文档状态与 devlog 收口。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/Cargo.toml`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/world-runtime/distributed-runtime.md`

## 状态
- 当前阶段：DNM-3 完成，进入 DNM-4。
- 下一步：执行最终回归并完成文档状态收口。
- 最近更新：2026-02-16。
