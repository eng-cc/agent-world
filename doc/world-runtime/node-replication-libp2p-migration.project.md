# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈（项目管理文档）

## 任务拆解
- [x] NRM-0：输出设计文档与项目管理文档。
- [x] NRM-1：Node 注入 `DistributedNetwork` 复制通道（优先网络，UDP 回退）并补测试。
- [x] NRM-2：增强外部注入接线（replication topic 配置与隔离测试）。
- [ ] NRM-3：执行回归测试、更新状态并收口。

## 依赖
- `crates/node`
- `crates/agent_world_proto`
- `doc/world-runtime/node-distfs-replication-network-closure.md`

## 状态
- 当前阶段：NRM-2 完成，进入 NRM-3。
- 下一步：执行收口回归并完成文档状态更新。
- 最近更新：2026-02-16。
