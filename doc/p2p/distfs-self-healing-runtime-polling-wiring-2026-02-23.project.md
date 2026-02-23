# Agent World Runtime：分布式存储自愈轮询 Runtime 接线（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/p2p/distfs-self-healing-runtime-polling-wiring-2026-02-23.md`
- [x] 项目文档：`doc/p2p/distfs-self-healing-runtime-polling-wiring-2026-02-23.project.md`

### T1 Runtime 接线实现
- [ ] 新增 Node 级副本维护配置模型与校验
- [ ] 增加 Runtime 轮询状态字段与 tick 调用链接线
- [ ] 增加 Node 侧本地目标执行器（网络拉取 + 本地 CAS 写入）
- [ ] 补齐单测（启用执行/缺失依赖跳过/非法配置）

### T2 收口
- [ ] 回归：`agent_world_node`、`agent_world_net`、`agent_world_distfs`、`agent_world_consensus`
- [ ] 更新设计/项目文档状态
- [ ] 追加 `doc/devlog/2026-02-23.md` 任务日志

## 依赖
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/node_runtime_core.rs`
- `crates/agent_world_node/src/types.rs`
- `crates/agent_world_node/src/network_bridge.rs`
- `crates/agent_world_node/src/tests_split_part1.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2
- 阻塞项：无
