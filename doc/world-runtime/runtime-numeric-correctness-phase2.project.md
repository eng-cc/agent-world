# Agent World Runtime：共识数值语义与原子状态转移硬化（15 点清单第二阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase2.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase2.project.md`

### T1 Node Points 账本显式溢出语义
- [x] `NodePointsLedger::settle_epoch` 改为可失败接口，移除关键路径静默饱和。
- [x] `NodePointsRuntimeCollector` 结算路径透传错误，避免吞错。
- [x] 增加溢出拒绝与原子性测试。

### T2 Node PoS 票权与 slot 递进显式溢出语义
- [x] `insert_attestation` 改为 `checked_add` 并返回可观测错误。
- [x] `propose_next_head` 的 `next_slot` 递进改为 `checked_add`。
- [x] 增加 PoS 溢出拒绝测试并保证失败不污染 pending 状态。

### T3 回归与收口
- [x] 运行对应 `test_tier_required` 口径的定向测试。
- [x] 更新设计/项目文档状态与 `doc/devlog/2026-02-22.md`。

## 依赖
- Runtime：
  - `crates/agent_world/src/runtime/node_points.rs`
  - `crates/agent_world/src/runtime/node_points_runtime.rs`
- Consensus/Node：
  - `crates/agent_world_consensus/src/node_pos.rs`
  - `crates/agent_world_consensus/src/pos.rs`
  - `crates/agent_world_node/src/lib.rs`
- 测试：
  - `crates/agent_world/src/runtime/node_points.rs`（内联 tests）
  - `crates/agent_world/src/runtime/node_points_runtime.rs`（内联 tests）
  - `crates/agent_world_consensus/src/pos.rs`（内联 tests）

## 状态
- 当前状态：`已完成`
- 已完成：T0、T1、T2、T3
- 进行中：无
- 未开始：无
- 阻塞项：无
