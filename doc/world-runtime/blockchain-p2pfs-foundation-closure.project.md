# Agent World Runtime：基础区块链 + P2P FS 三步收敛（项目管理文档）

## 任务拆解
- [x] BPFS-0：输出设计文档与项目管理文档。
- [x] BPFS-1：落地 Sequencer Mainloop（action->mempool->batch->pos->dht）与单元测试。
- [x] BPFS-2：接入签名/验签最小闭环（提交校验、拒绝路径、测试）。
- [x] BPFS-3：落地 DistFS 最小跨节点复制一致（单写者冲突保护 + 回放恢复测试）。

## 依赖
- `crates/agent_world_consensus`
- `crates/agent_world_net`
- `crates/agent_world_distfs`
- `doc/world-runtime/distributed-runtime.md`
- `doc/world-runtime/distributed-pos-consensus.md`

## 状态
- 当前阶段：BPFS-1 ~ BPFS-3 全部完成。
- 下一步：将 `FileReplicationRecord` 与网络分发链路打通（node/net 层复制消息接线）。
- 最近更新：2026-02-16。
