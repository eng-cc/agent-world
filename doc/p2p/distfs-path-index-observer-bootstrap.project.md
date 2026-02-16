# Agent World Runtime：Observer/Bootstrap 路径索引读取接入（项目管理文档）

## 任务拆解
- [x] POBI-1：设计文档与项目管理文档落地。
- [x] POBI-2：实现 bootstrap/head_follow/observer 的路径索引入口。
- [x] POBI-3：补齐单元测试并完成 `agent_world_net` 回归。
- [x] POBI-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world_net/src/bootstrap.rs`
- `crates/agent_world_net/src/head_follow.rs`
- `crates/agent_world_net/src/observer.rs`
- `crates/agent_world_net/src/execution_storage.rs`
- `doc/p2p/distfs-runtime-path-index.md`

## 状态
- 当前阶段：Observer/Bootstrap 路径索引读取接入完成（POBI-1~POBI-4 全部完成）。
- 下一步：将路径索引模式与网络模式在同一 observer 跟随循环中的策略切换能力显式化（配置层）。
- 最近更新：2026-02-16。
