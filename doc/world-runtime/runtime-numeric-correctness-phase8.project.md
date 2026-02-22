# Agent World Runtime：Membership Dead-Letter Replay 重试计数与比率阈值数值语义硬化（15 点清单第八阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase8.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase8.project.md`

### T1 重试递进受检语义
- [x] `membership_recovery/types.rs` 将 `with_retry_failure` 改为受检递进语义。
- [x] `membership_recovery/mod.rs` 接线可失败路径，确保越界失败不发生部分写入。
- [x] 新增重试计数/回退时间越界测试。

### T2 自适应策略阈值/比率去饱和化
- [ ] `membership_recovery/replay.rs` 的 `* 2` 阈值比较与 `* 1000` 比率换算改为无溢出实现。
- [ ] 新增极端大整数边界测试，验证策略判断不被饱和乘法污染。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_recovery/types.rs`
- `crates/agent_world_consensus/src/membership_recovery/mod.rs`
- `crates/agent_world_consensus/src/membership_recovery/replay.rs`
- `crates/agent_world_consensus/src/membership_recovery_tests.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3
- 阻塞项：无
