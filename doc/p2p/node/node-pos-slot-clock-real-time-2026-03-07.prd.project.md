# Agent World Runtime：PoS 固定时间槽（Slot/Epoch）真实时钟驱动（项目管理文档）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-P2P-008-T0 (PRD-P2P-NODE-CLOCK-001/002/003) [test_tier_required]: 完成专题 PRD 与项目管理建档，并回写 `doc/p2p/prd.md` / `doc/p2p/prd.project.md` / `doc/p2p/prd.index.md`。
- [x] TASK-P2P-008-T1 (PRD-P2P-NODE-CLOCK-001) [test_tier_required]: 在 `NodePosConfig/PosNodeEngine` 引入 wall-clock slot 计算、漏槽对齐与提案门控。
- [ ] TASK-P2P-008-T2 (PRD-P2P-NODE-CLOCK-002/003) [test_tier_required]: 增加时间窗口校验与快照可观测字段（`last_observed_slot`、`missed_slot_count`）。当前进展：快照可观测字段已完成，入站时间窗口校验待实现。
- [ ] TASK-P2P-008-T3 (PRD-P2P-NODE-CLOCK-001/002/003) [test_tier_required + test_tier_full]: 补齐单元/跨节点回归，回写文档与 devlog 收口。

## 依赖
- `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
- `crates/agent_world_node/src/types.rs`
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/lib_impl_part1.rs`
- `crates/agent_world_node/src/lib_impl_part2.rs`
- `crates/agent_world_node/src/pos_state_store.rs`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-07
- 当前状态: active
- 下一任务: `TASK-P2P-008-T2`
- 阻塞项: 无
- 进展: 已完成 wall-clock slot/epoch 驱动、漏槽对齐、提案门控、重启单调恢复与 required-tier 定向回归。
- 说明: 本文档仅维护执行计划与任务状态；实施过程记录写入 `doc/devlog/2026-03-07.md`。
