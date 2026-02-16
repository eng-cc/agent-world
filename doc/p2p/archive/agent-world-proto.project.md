> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：`agent_world_proto` 协议类型与 Trait 抽离（项目管理文档）

## 任务拆解
- [x] T1：新增设计文档与项目管理文档（本文件）。
- [x] T2：新建 `crates/agent_world_proto`，迁移协议类型与 trait 定义。
- [x] T3：`agent_world` 侧适配（wrapper + 实现对接）并完成编译/测试回归。
- [x] T4：扩展设计/项目文档，纳入共识成员变更协议类型抽离范围。
- [x] T5：迁移共识成员变更协议类型到 `agent_world_proto` 并完成 `agent_world` 侧适配回归。

## 依赖
- `doc/p2p/distributed-runtime.md`
- `crates/agent_world/src/runtime/distributed.rs`
- `crates/agent_world/src/runtime/distributed_net.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`

## 状态
- 当前阶段：T1~T5 全部完成。
- 下一步：按需继续把跨节点传输的协议数据结构收敛到 `agent_world_proto`，保持 `agent_world` 侧仅承载实现逻辑。
- 最近更新：2026-02-12
