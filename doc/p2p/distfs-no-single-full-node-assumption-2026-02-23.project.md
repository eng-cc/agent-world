# Agent World Runtime：分布式存储去单机完整依赖改造（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/p2p/distfs-no-single-full-node-assumption-2026-02-23.md`
- [x] 项目文档：`doc/p2p/distfs-no-single-full-node-assumption-2026-02-23.project.md`

### T1 严格 DHT 拉取
- [x] `fetch_blob_from_dht` 去掉无 provider 回退路径
- [x] 增补 `agent_world_net` 单测：无 provider 失败 / provider 重试成功

### T2 覆盖审计与回放接线
- [ ] 新增 provider 覆盖审计策略模块
- [ ] 在 `replay_validate_with_head_and_dht` 接入覆盖审计
- [ ] 补齐单测：副本不足拒绝、单节点全覆盖拒绝、分布覆盖放行

### T3 收口
- [ ] 回归：`agent_world_net`、`agent_world_distfs`、`agent_world_consensus`、`agent_world_node`
- [ ] 更新设计/项目文档状态
- [ ] 追加 `doc/devlog/2026-02-23.md` 任务日志

## 依赖
- `crates/agent_world_net/src/client.rs`
- `crates/agent_world_net/src/observer_replay.rs`
- `crates/agent_world_net/src/replay_flow.rs`
- `crates/agent_world_net/src/tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
