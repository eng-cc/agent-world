# Agent World Runtime：Sequencer 主循环与 Lease 递进数值语义硬化（15 点清单第五阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase5.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase5.project.md`

### T1 Sequencer 递进语义硬化
- [ ] `sequencer_mainloop` 的 `next_slot`/`next_height` 递进改为受检溢出语义。
- [ ] 终态更新改为“先计算后提交”，失败时避免局部状态写入。
- [ ] 新增对应溢出拒绝测试。

### T2 Lease 递进语义硬化
- [ ] `lease` 的 `next_term` 与 `expires_at_ms` 递进改为受检语义。
- [ ] acquire/renew 越界时拒绝并保持原状态。
- [ ] 新增对应溢出拒绝测试。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/sequencer_mainloop.rs`
- `crates/agent_world_consensus/src/lease.rs`
- `crates/agent_world_consensus/src/lib.rs`
- `doc/world-runtime/runtime-numeric-correctness-phase5.md`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3
- 阻塞项：无
