# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 1）项目管理文档

## 任务拆解
- [x] HP1-0：输出设计文档与项目管理文档。
- [x] HP1-1：`world_viewer_live` 注入 libp2p replication 网络（CLI + 启动接线 + 测试）。
- [x] HP1-2：Node PoS gossip 扩展 proposal/attestation（广播 + 消费 + 回归）。
- [x] HP1-3：执行回归测试，更新文档状态与 devlog 收口。

## 依赖
- `crates/agent_world`
- `crates/agent_world_node`
- `crates/agent_world_net`
- `doc/world-runtime/node-replication-libp2p-migration.md`
- `doc/world-runtime/distributed-node-pos-gossip.md`

## 状态
- 当前阶段：HP1-0~HP1-3 全部完成。
- 下一步：进入 Phase 2（Action/Head 节点签名链路 + Node PoS 状态持久化 + observer 运行态面板接线）。 
- 最近更新：2026-02-16。
