# Agent World Runtime：Node DistFS 复制网络化收敛（项目管理文档）

## 任务拆解
- [x] NRX-0：输出设计文档与项目管理文档。
- [x] NRX-1：node gossip 接入 DistFS 复制消息（广播+应用+guard 持久化）。
- [x] NRX-2：接入复制消息签名/验签（消费 config.toml 节点密钥）。
- [x] NRX-3：补齐多节点复制与重启恢复测试，完成收口。

## 依赖
- `crates/agent_world_node`
- `crates/agent_world_distfs`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `doc/world-runtime/blockchain-p2pfs-foundation-closure.md`
- `doc/world-runtime/node-keypair-config-bootstrap.md`

## 状态
- 当前阶段：NRX-0 ~ NRX-3 全部完成。
- 下一步：将当前 UDP gossip 复制路径逐步迁移到 `agent_world_net/libp2p` 统一网络栈。
- 最近更新：2026-02-16。
