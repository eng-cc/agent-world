# Agent World Runtime：PoS 超多数比率边界数值语义硬化（15 点清单第七阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase7.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase7.project.md`

### T1 PoS 比率判定去饱和化
- [x] `crates/agent_world_proto/src/distributed_pos.rs` 超多数比率判定改为无溢出实现。
- [x] `crates/agent_world_consensus/src/pos.rs` 同步改造。
- [x] `crates/agent_world_node/src/pos_validation.rs` 同步改造。

### T2 边界测试补齐
- [ ] 新增大整数比率边界测试（`denominator = u64::MAX`）验证不会误拒绝。
- [ ] 校验 proto/consensus/node 三层行为一致。

### T3 回归与收口
- [ ] 运行 `agent_world_proto`、`agent_world_consensus`、`agent_world_node` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_proto/src/distributed_pos.rs`
- `crates/agent_world_consensus/src/pos.rs`
- `crates/agent_world_node/src/pos_validation.rs`
- `crates/agent_world_node/src/tests_hardening.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
