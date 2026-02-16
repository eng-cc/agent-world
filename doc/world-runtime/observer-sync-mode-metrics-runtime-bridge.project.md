# Agent World Runtime：Observer 同步源统计桥接（项目管理文档）

## 任务拆解
- [x] OSMB-1：设计文档与项目管理文档落地。
- [ ] OSMB-2：实现桥接接口与导出。
- [ ] OSMB-3：补齐桥接接口测试并完成 `agent_world_net` 回归。
- [ ] OSMB-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world_net/src/observer.rs`
- `crates/agent_world_net/src/observer_metrics.rs`
- `crates/agent_world_net/src/lib.rs`
- `doc/world-runtime/observer-sync-mode-runtime-metrics.md`

## 状态
- 当前阶段：OSMB-1 完成，进入 OSMB-2（桥接接口实现）。
- 下一步：新增自动记录 metrics 的 sync/follow 入口并导出。
- 最近更新：2026-02-16。
