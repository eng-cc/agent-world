# Agent World Runtime：分布式存储自愈控制面（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/p2p/distfs-self-healing-control-plane-2026-02-23.md`
- [x] 项目文档：`doc/p2p/distfs-self-healing-control-plane-2026-02-23.project.md`

### T1 维护计划生成器
- [x] 新增 `replica_maintenance` 计划模型与策略
- [x] 实现 Repair/Rebalance 任务规划
- [x] 补齐单测：副本不足与负载倾斜场景

### T2 维护执行器
- [x] 增加计划执行接口与执行报告
- [x] 执行成功后发布 target provider 到 DHT
- [x] 补齐单测：成功发布索引 / 失败不污染索引

### T3 收口
- [x] 回归：`agent_world_net`、`agent_world_distfs`、`agent_world_consensus`、`agent_world_node`
- [x] 更新设计/项目文档状态
- [x] 追加 `doc/devlog/2026-02-23.md` 任务日志

## 依赖
- `crates/agent_world_net/src/lib.rs`
- `crates/agent_world_net/src/provider_selection.rs`
- `crates/agent_world_net/src/dht.rs`
- `crates/agent_world_net/src/tests.rs`

## 状态
- 当前状态：`已完成`
- 已完成：T0、T1、T2、T3
- 进行中：无
- 未开始：无
- 阻塞项：无
