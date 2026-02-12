# Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分（项目管理文档）

## 任务拆解
- [x] T1：新增设计文档与项目管理文档（本文件）。
- [x] T2：新建 `agent_world_net` / `agent_world_consensus` crate，并接入 workspace。
- [x] T3：完成 net/consensus 导出能力面，确保 `distributed_membership_sync` 纳入 `agent_world_consensus`。
- [x] T4：完成编译与定向测试回归，回写文档状态并收口。
- [x] T5：将 `distributed_net` 核心实现下沉到 `agent_world_net`（`InMemoryNetwork` 与网络 trait/type）。
- [x] T6：完成扩展阶段回归验证与文档收口。
- [ ] T7：将 `distributed_dht` 核心实现下沉到 `agent_world_net`（`InMemoryDht` 与 DHT trait/type）。
- [ ] T8：完成二次扩展阶段回归验证与文档收口。

## 依赖
- `crates/agent_world/src/runtime/mod.rs`
- `crates/agent_world/src/runtime/distributed_net.rs`
- `crates/agent_world/src/runtime/libp2p_net.rs`
- `crates/agent_world/src/runtime/distributed_consensus.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`
- `crates/agent_world/src/runtime/distributed_dht.rs`
- `crates/agent_world_net/src/lib.rs`

## 状态
- 当前阶段：T6 完成，T7 进行中。
- 下一步：完成 `distributed_dht` 迁移并执行二次扩展阶段回归。
- 最近更新：2026-02-12
