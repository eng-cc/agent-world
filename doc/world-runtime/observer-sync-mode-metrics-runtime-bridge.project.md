# Agent World Runtime：Observer 同步源统计桥接（项目管理文档）

## 任务拆解
- [x] OSMB-1：设计文档与项目管理文档落地。
- [x] OSMB-2：实现桥接接口与导出。
- [ ] OSMB-3：补齐桥接接口测试并完成 `agent_world_net` 回归。
- [ ] OSMB-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world_net/src/observer.rs`
- `crates/agent_world_net/src/observer_metrics.rs`
- `crates/agent_world_net/src/lib.rs`
- `doc/world-runtime/observer-sync-mode-runtime-metrics.md`

## 状态
- 当前阶段：OSMB-2 完成，进入 OSMB-3（桥接接口测试）。
- 下一步：补齐同步/跟随桥接接口测试，验证自动计数语义与回退统计。
- 最近更新：2026-02-16。
