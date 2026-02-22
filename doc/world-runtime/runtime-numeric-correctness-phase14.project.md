# Agent World Runtime：Membership Sync 与 Mempool 批处理计数算术语义硬化（15 点清单第十四阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase14.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase14.project.md`

### T1 Membership Sync 计数受检化
- [x] `sync_key_revocations`、`sync_key_revocations_with_policy`、`sync_membership_directory` 关键计数改为受检累加。
- [x] 增加计数 helper overflow 测试并验证错误语义一致。

### T2 Mempool payload 累加受检化
- [ ] `take_batch_with_rules` 的 `total_bytes + size_bytes` 改为受检累加。
- [ ] 增加 overflow 测试并验证失败路径不移除 action。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership.rs`
- `crates/agent_world_consensus/src/mempool.rs`
- `crates/agent_world_consensus/src/membership_tests.rs`
- `crates/agent_world_consensus/src/mempool.rs`（模块内测试）

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
