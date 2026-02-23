# Agent World Runtime：异构节点分布式存储最优稳定性改造（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/p2p/distfs-heterogeneous-node-optimal-stability-2026-02-23.md`
- [x] 项目文档：`doc/p2p/distfs-heterogeneous-node-optimal-stability-2026-02-23.project.md`

### T1 Provider 能力画像扩展
- [x] 扩展 `ProviderRecord` 可选能力字段并保持 serde 向后兼容
- [x] 更新 `agent_world_net` / `agent_world_consensus` 相关 DHT provider 构造路径
- [x] 补齐兼容测试

### T2 评分排序与重试拉取
- [ ] 新增 provider 评分策略模块（权重 + 归一化）
- [ ] `DistributedClient::fetch_blob_from_dht` 升级为排序后逐 provider 重试
- [ ] 补齐 `agent_world_net` 单测（排序、重试、回退）

### T3 收口
- [ ] 运行回归：`agent_world_net`、`agent_world_distfs`、`agent_world_consensus`、`agent_world_node`
- [ ] 更新设计/项目文档状态
- [ ] 追加 `doc/devlog/2026-02-23.md` 任务日志

## 依赖
- `crates/agent_world_proto/src/distributed_dht.rs`
- `crates/agent_world_net/src/client.rs`
- `crates/agent_world_net/src/dht.rs`
- `crates/agent_world_net/src/dht_cache.rs`
- `crates/agent_world_net/src/provider_cache.rs`
- `crates/agent_world_net/src/libp2p_net.rs`
- `crates/agent_world_net/src/tests.rs`
- `crates/agent_world_consensus/src/dht.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
