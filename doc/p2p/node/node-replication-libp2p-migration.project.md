# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈（项目管理文档）

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] NRM-0 (PRD-P2P-MIG-099)：输出设计文档与项目管理文档。
- [x] NRM-1 (PRD-P2P-MIG-099)：Node 注入 `DistributedNetwork` 复制通道（优先网络，UDP 回退）并补测试。
- [x] NRM-2 (PRD-P2P-MIG-099)：增强外部注入接线（replication topic 配置与隔离测试）。
- [x] NRM-3 (PRD-P2P-MIG-099)：执行回归测试、更新状态并收口。
- [x] NRM-4 (PRD-P2P-MIG-099)：统一 crate 目录路径为 `crates/agent_world_node`，消除路径歧义。

## 依赖
- doc/p2p/node/node-replication-libp2p-migration.prd.md
- `crates/agent_world_node`
- `crates/agent_world_proto`
- `doc/p2p/node/node-distfs-replication-network-closure.prd.md`

## 状态
- 当前阶段：NRM-0 ~ NRM-4 全部完成。
- 下一步：在上层集成（独立于 `agent_world` 依赖环）接入具体 libp2p 网络实现并做端到端联调。
- 最近更新：2026-02-16。
