# Agent World Runtime：`agent_world_proto` 协议类型与 Trait 抽离（项目管理文档）

## 任务拆解
- [x] T1：新增设计文档与项目管理文档（本文件）。
- [x] T2：新建 `crates/agent_world_proto`，迁移协议类型与 trait 定义。
- [x] T3：`agent_world` 侧适配（wrapper + 实现对接）并完成编译/测试回归。
- [x] T4：扩展设计/项目文档，纳入共识成员变更协议类型抽离范围。
- [ ] T5：迁移共识成员变更协议类型到 `agent_world_proto` 并完成 `agent_world` 侧适配回归。

## 依赖
- `doc/world-runtime/distributed-runtime.md`
- `crates/agent_world/src/runtime/distributed.rs`
- `crates/agent_world/src/runtime/distributed_net.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`

## 状态
- 当前阶段：T4 完成，T5 进行中。
- 下一步：完成 `distributed_consensus` 协议类型迁移并跑回归。
- 最近更新：2026-02-12
