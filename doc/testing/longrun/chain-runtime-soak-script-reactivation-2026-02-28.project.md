# oasis7: 基于 oasis7_chain_runtime 的长跑脚本可用化（2026-02-28）（项目管理）

- 对应设计文档: `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.design.md`
- 对应需求文档: `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.md`

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] SOAKREACT-1 (PRD-TESTING-LONGRUN-SOAK-001/002): 完成专题设计文档与项目管理文档建档。
- [x] SOAKREACT-2 (PRD-TESTING-LONGRUN-SOAK-001/002): 改造 `s10-five-node-game-soak.sh` 为可运行的 `oasis7_chain_runtime` 五节点脚本。
- [x] SOAKREACT-3 (PRD-TESTING-LONGRUN-SOAK-002/003): 改造 `p2p-longrun-soak.sh` 为可运行的 `oasis7_chain_runtime` 脚本并保留 chaos 注入。
- [x] SOAKREACT-4 (PRD-TESTING-LONGRUN-SOAK-002/003): 完成回归验证、手册口径与任务日志收口。
- [x] SOAKREACT-5 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.project.md`。
- [x] SOAKREACT-6 (PRD-TESTING-LONGRUN-SOAK-002): 采样字段口径补充 `worker_poll_count` 与 `consensus.last_observed_tick`，明确与共识进度的语义边界。

## 依赖
- doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.md
- `scripts/s10-five-node-game-soak.sh`
- `scripts/p2p-longrun-soak.sh`
- `crates/oasis7/src/bin/oasis7_chain_runtime.rs`
- `testing-manual.md`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-08
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
