# Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分（项目管理文档）

## 任务拆解
- [x] T1：新增设计文档与项目管理文档（本文件）。
- [x] T2：新建 `agent_world_net` / `agent_world_consensus` crate，并接入 workspace。
- [x] T3：完成 net/consensus 导出能力面，确保 `distributed_membership_sync` 纳入 `agent_world_consensus`。
- [ ] T4：完成编译与定向测试回归，回写文档状态并收口。

## 依赖
- `crates/agent_world/src/runtime/mod.rs`
- `crates/agent_world/src/runtime/distributed_net.rs`
- `crates/agent_world/src/runtime/libp2p_net.rs`
- `crates/agent_world/src/runtime/distributed_consensus.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync.rs`

## 状态
- 当前阶段：T3 完成，T4 进行中。
- 下一步：执行回归检查并完成文档收口。
- 最近更新：2026-02-12
