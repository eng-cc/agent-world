# Agent World Runtime：Observer 同步源策略化（项目管理文档）

## 任务拆解
- [x] OSSM-1：设计文档与项目管理文档落地。
- [x] OSSM-2：实现 `HeadSyncSourceMode` 与 `ObserverClient` 模式化同步入口。
- [ ] OSSM-3：补齐单元测试并完成 `agent_world_net` 回归。
- [ ] OSSM-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world_net/src/observer.rs`
- `crates/agent_world_net/src/head_follow.rs`
- `crates/agent_world_net/src/lib.rs`
- `doc/world-runtime/distfs-path-index-observer-bootstrap.md`

## 状态
- 当前阶段：OSSM-1~OSSM-2 已完成（策略枚举与模式化接口已实现）。
- 下一步：OSSM-3（补齐模式测试并回归）。
- 最近更新：2026-02-16。
