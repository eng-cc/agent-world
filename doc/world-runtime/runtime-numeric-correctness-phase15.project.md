# Agent World Runtime：Membership Recovery Federated 聚合扫描与游标计数算术语义硬化（15 点清单第十五阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase15.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase15.project.md`

### T1 Federated 聚合扫描计数受检化
- [x] `scanned_hot` / `scanned_cold` 从饱和累加改为受检累加。
- [x] 增加对应 overflow 单元测试并验证错误语义一致。

### T2 复合游标偏移计数受检化
- [ ] `node_event_offset` 递进从饱和递进改为受检递进。
- [ ] 增加对应 overflow 单元测试并验证错误语义一致。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_recovery/replay_archive_federated.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_archive_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
