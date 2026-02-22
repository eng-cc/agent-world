# Agent World Runtime：Governance Drill/Retention 时间算术数值语义硬化（15 点清单第九阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase9.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase9.project.md`

### T1 调度与保留时间算术受检化
- [ ] `membership_recovery/replay_archive.rs` 的 drill schedule 时间算术改为受检语义。
- [ ] `membership_recovery/replay_archive.rs` 的 audit retention 年龄计算改为受检语义。
- [ ] 新增 schedule/retention 溢出拒绝测试并验证无状态污染。

### T2 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_recovery/replay_archive.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_archive_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2
- 阻塞项：无
