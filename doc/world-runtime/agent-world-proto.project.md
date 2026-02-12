# Agent World Runtime：`agent_world_proto` 协议类型与 Trait 抽离（项目管理文档）

## 任务拆解
- [x] T1：新增设计文档与项目管理文档（本文件）。
- [ ] T2：新建 `crates/agent_world_proto`，迁移协议类型与 trait 定义。
- [ ] T3：`agent_world` 侧适配（wrapper + 实现对接）并完成编译/测试回归。

## 依赖
- `doc/world-runtime/distributed-runtime.md`
- `crates/agent_world/src/runtime/distributed.rs`
- `crates/agent_world/src/runtime/distributed_net.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`

## 状态
- 当前阶段：T1 完成，T2 进行中。
- 下一步：创建 `agent_world_proto` crate 并迁移协议类型、trait。
- 最近更新：2026-02-12
