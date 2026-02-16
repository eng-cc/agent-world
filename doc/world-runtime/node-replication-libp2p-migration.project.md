# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈（项目管理文档）

## 任务拆解
- [x] NRM-0：输出设计文档与项目管理文档。
- [ ] NRM-1：Node 注入 `DistributedNetwork` 复制通道（优先网络，UDP 回退）并补测试。
- [ ] NRM-2：world_viewer_live 接入 replication libp2p 配置与启动。
- [ ] NRM-3：执行回归测试、更新状态并收口。

## 依赖
- `crates/node`
- `crates/agent_world_net`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `doc/world-runtime/node-distfs-replication-network-closure.md`

## 状态
- 当前阶段：NRM-0 完成，进入 NRM-1。
- 下一步：在 node runtime 增加统一网络复制 endpoint。
- 最近更新：2026-02-16。
