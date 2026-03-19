# oasis7: 发布门禁指标策略对齐（2026-02-28）（项目管理）

- 对应设计文档: `doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.design.md`
- 对应需求文档: `doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] RELGATE-1 (PRD-TESTING-GOV-RELEASE-001/003): 完成专题设计文档与项目管理文档建档。
- [x] RELGATE-2 (PRD-TESTING-GOV-RELEASE-001/002): 接通 `world_chain_runtime` reward runtime worker 与 `/v1/chain/status` 指标输出。
- [x] RELGATE-3 (PRD-TESTING-GOV-RELEASE-002/003): S9/S10 脚本切换到真实 reward runtime 指标并修正 chaos 误判。
- [x] RELGATE-4 (PRD-TESTING-GOV-RELEASE-003): 完成回归验证、手册同步、devlog 与项目文档收口。
- [x] RELGATE-5 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.project.md`。

## 依赖
- doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.prd.md
- Runtime:
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/reward_runtime_worker.rs`
- 长跑脚本：
  - `scripts/p2p-longrun-soak.sh`
  - `scripts/s10-five-node-game-soak.sh`
- 手册与追踪：
  - `testing-manual.md`
  - `doc/testing/prd.md`
  - `doc/testing/project.md`
- 回归产物样例：
  - `.tmp/release_gate_s10/20260228-222029/summary.json`
  - `.tmp/release_gate_p2p/20260228-225152/summary.json`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
