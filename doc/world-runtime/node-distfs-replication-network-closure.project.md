# Agent World Runtime：Node DistFS 复制网络化收敛（项目管理文档）

## 任务拆解
- [x] NRX-0：输出设计文档与项目管理文档。
- [ ] NRX-1：node gossip 接入 DistFS 复制消息（广播+应用+guard 持久化）。
- [ ] NRX-2：接入复制消息签名/验签（消费 config.toml 节点密钥）。
- [ ] NRX-3：补齐多节点复制与重启恢复测试，完成收口。

## 依赖
- `crates/node`
- `crates/agent_world_distfs`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `doc/world-runtime/blockchain-p2pfs-foundation-closure.md`
- `doc/world-runtime/node-keypair-config-bootstrap.md`

## 状态
- 当前阶段：NRX-0 完成，进入 NRX-1。
- 下一步：在 node gossip 主循环中接入 replication 消息与应用流程。
- 最近更新：2026-02-16。
