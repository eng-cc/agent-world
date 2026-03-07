# p2p PRD Project

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-P2P-001 (PRD-P2P-001) [test_tier_required]: 完成 p2p PRD 改写，建立分布式系统设计入口。
- [ ] TASK-P2P-002 (PRD-P2P-001/002) [test_tier_required]: 补齐网络/共识/DistFS 三线联合验收清单。
- [ ] TASK-P2P-003 (PRD-P2P-002/003) [test_tier_required]: 建立 S9/S10 长跑结果与缺陷闭环模板。
- [ ] TASK-P2P-004 (PRD-P2P-003) [test_tier_required]: 对接发行门禁中的分布式质量指标。
- [x] TASK-P2P-005 (PRD-P2P-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-P2P-006 (PRD-P2P-004) [test_tier_required]: 输出“手机轻客户端权威状态”专题 PRD 与项目管理文档，并回写模块主索引链路。
- [ ] TASK-P2P-007 (PRD-P2P-004) [test_tier_required + test_tier_full]: 实现 intent-only 接入、finality UI、challenge/reconnect 闭环并补齐回归证据。
- [x] TASK-P2P-008 (PRD-P2P-005) [test_tier_required + test_tier_full]: 实现 PoS 固定时间槽（slot/epoch）真实时钟驱动、漏槽计数与时间窗口校验，并补齐回归证据。
- [ ] TASK-P2P-009 (PRD-P2P-006) [test_tier_required + test_tier_full]: 实现 PoS 槽内 tick 相位门控（`ticks_per_slot`）与动态节拍调度，并补齐回归证据。

## 依赖
- doc/p2p/prd.index.md
- `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
- `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`
- `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
- `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-07
- 当前状态: active
- 下一任务: TASK-P2P-009-T1 / TASK-P2P-002
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 本轮新增: `TASK-P2P-006` 已完成，专题文档 `p2p-mobile-light-client-authoritative-state-2026-03-06` 已纳入索引和模块追踪映射。
- 本轮新增: `TASK-P2P-008` 已建档，专题文档 `node-pos-slot-clock-real-time-2026-03-07` 已纳入模块追踪映射。
- 本轮新增: `TASK-P2P-009` 已建档，专题文档 `node-pos-subslot-tick-pacing-2026-03-07` 已纳入模块追踪映射。
- TASK-P2P-007 进展（2026-03-07）: 已完成子任务 `TASK-P2P-MLC-002`（intent `tick/seq/sig` 字段、`runtime_live` 幂等 ACK、相关回归测试）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-003` 已完成代码落地与定向 required 回归（权威批次 `state_root/data_root`、`pending/confirmed/final` 状态机、final-only 结算/排行闸门）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-004` 已完成代码落地与 `test_tier_full` 定向回归（watcher challenge 入口、resolve/slash 仲裁、challenge 阻断 final）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-005` 已完成代码落地与定向 required 回归（稳定点回滚、重连追平元数据、会话吊销换钥与鉴权拦截）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-006` 已完成 required/full 联合回归与门禁证据沉淀，MLC 专题任务全部收口。
- TASK-P2P-008 进展（2026-03-07）: `TASK-P2P-008-T0~T3` 全部完成（`slot_duration_ms`/`slot_clock_genesis_unix_ms`、wall-clock 提案门控、`last_observed_slot`/`missed_slot_count` 持久化、proposal/attestation 未来槽与过旧槽拒绝、attestation epoch 映射校验、拒绝原因快照可观测、跨节点 gossip/network 定向回归）。
- TASK-P2P-009 进展（2026-03-07）: `TASK-P2P-009-T0` 已完成（专题 PRD/项目管理建档，明确 `10 tick/slot` 相位门控与动态调度方案）。
- TASK-P2P-009 进展（2026-03-07）: `TASK-P2P-009-T1` 已完成代码落地与 required 回归（`ticks_per_slot/proposal_tick_phase` 配置校验、logical tick 观测、提案相位门控、tick 级快照持久化）。
- 说明: 本文档仅维护 p2p 设计执行状态；过程记录在 `doc/devlog/2026-03-07.md`。
- ROUND-002 进展（2026-03-05）: 已并行完成 `B3-C2-009-S2/C2-010/C2-011`（observer sync-mode、node-contribution、distfs-self-healing）主从化回写。
- ROUND-002 进展（2026-03-05）: 已并行完成 `B3-C2-003/C2-008-S1/C2-008-S2`（node-redeemable-power-asset、distfs-production-hardening phase1~9）主从化回写。
