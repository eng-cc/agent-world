# Agent World Runtime：分布式存储自愈定时轮询（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/p2p/distfs-self-healing-polling-loop-2026-02-23.md`
- [x] 项目文档：`doc/p2p/distfs-self-healing-polling-loop-2026-02-23.project.md`

### T1 轮询能力实现
- [x] 新增轮询策略/状态/结果模型
- [x] 实现按间隔触发的轮询入口（到期执行 plan+execute）
- [x] 补齐单测：首轮执行/间隔未到跳过/非法策略

### T2 收口
- [ ] 回归：`agent_world_net`、`agent_world_distfs`、`agent_world_consensus`、`agent_world_node`
- [ ] 更新设计/项目文档状态
- [ ] 追加 `doc/devlog/2026-02-23.md` 任务日志

## 依赖
- `crates/agent_world_net/src/replica_maintenance.rs`
- `crates/agent_world_net/src/lib.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：无
- 阻塞项：无
