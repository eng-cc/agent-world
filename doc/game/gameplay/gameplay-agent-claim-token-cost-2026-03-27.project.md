# Gameplay Agent 认领代币成本与维护机制（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.prd.md`

审计轮次: 1

## 任务拆解

- [x] TASK-GAMEPLAY-AGC-001 (`PRD-GAME-011`) [test_tier_required]: `producer_system_designer` 已建立 agent claim 成本专题，冻结“首个也不免费”的规则边界、三段式成本结构、回收条件与 root 文档挂载。
- [x] TASK-GAMEPLAY-AGC-002 (`PRD-GAME-011`) [test_tier_required + test_tier_full]: `runtime_engineer` 已落地 canonical claim 状态机、main token 扣费/锁定/退款/惩罚记账、epoch upkeep 结算与事件审计，并补齐 claim/release/grace/reclaim 的 required 定向回归。
- [x] TASK-GAMEPLAY-AGC-003 (`PRD-GAME-011`) [test_tier_required]: `viewer_engineer` 已把 canonical claim 概览接入 `player_gameplay.agent_claim`，补齐 pure API `--player-gameplay-only` 所需的未认领报价、已认领状态、cooldown / grace / idle reclaim 倒计时与 cap 阻断原因，并在 viewer 选中 agent 详情中落地对应文本。
- [x] TASK-GAMEPLAY-AGC-004 (`PRD-GAME-011`) [test_tier_required + test_tier_full]: `qa_engineer` 已建立 claim 并发、欠费、闲置、cap、refund/slash 与经济审计回归矩阵，并产出 `doc/testing/evidence/game-agent-claim-abuse-matrix-2026-03-27.md`。
- [ ] TASK-GAMEPLAY-AGC-005 (`PRD-GAME-011`) [test_tier_required]: `producer_system_designer` 基于首轮平衡数据复核 `slot multiplier / grace_epochs / penalty_bps / tier cap`，决定继续维持或新开调参专题。

## 依赖

- `doc/game/gameplay/gameplay-engineering-architecture.md`
- `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
- `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md`
- `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
- `testing-manual.md`

## 状态

- 更新日期: 2026-03-27
- 当前状态: in_progress
- 当前 owner: `producer_system_designer`
- 下一任务: `TASK-GAMEPLAY-AGC-005`
- 已完成补充:
  - `TASK-GAMEPLAY-AGC-001` 已新增 `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.{prd,design,project}.md`，并将 `PRD-GAME-011` 挂入 game 根 PRD / project / 索引 / README。
  - `TASK-GAMEPLAY-AGC-002` 已在 `crates/oasis7/src/runtime/` 落地 `ClaimAgent / ReleaseAgentClaim` 动作、claim 状态持久化、自动 upkeep/grace/idle reclaim processor 与 main token 账本联动。
  - `TASK-GAMEPLAY-AGC-003` 已在 `crates/oasis7/src/viewer/runtime_live/` 为 `player_gameplay` 增加 canonical `agent_claim` 快照，并在 `crates/oasis7_viewer/src/ui_text_claims.rs` / agent 详情文案中补齐 claim owner、状态、bond/upkeep、release/grace/forced reclaim 倒计时与未认领报价 blocker。
  - `TASK-GAMEPLAY-AGC-004` 已在 `crates/oasis7/src/runtime/tests/agent_claims.rs` 补齐并发单 owner 原子性、tier cap / slot 成本、grace 内恢复、release refund、欠费/闲置 reclaim slash-refund 对账断言，并把结果沉淀到 `doc/testing/evidence/game-agent-claim-abuse-matrix-2026-03-27.md`。
  - runtime v1 当前实现使用临时 base defaults：`activation fee=100`、`claim bond=200`、`upkeep=25`、`activation burn=50%`，并按 `reputation_score < 10 / >= 10 / >= 25` 映射 `tier-0 / tier-1 / tier-2+`；这些值供当前实现和测试闭环使用，后续仍由 `TASK-GAMEPLAY-AGC-005` 复核。
  - 本轮 required 验证已覆盖：首个 claim 非免费、重复认领拒绝、release cooldown refund、欠费 grace -> forced reclaim、idle warning -> forced reclaim。
  - 本轮 viewer / API required 验证已覆盖：
    - `env -u RUSTC_WRAPPER cargo check -p oasis7 --lib`
    - `env -u RUSTC_WRAPPER cargo check -p oasis7_viewer`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --lib compat_snapshot_exposes_player_agent_claim_overview -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7_viewer update_ui_populates_agent_selection_details_with_claim_state -- --nocapture`
  - 本轮 QA required/full 验证已覆盖：
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --lib --features test_tier_required runtime::tests::agent_claims:: -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --lib --features test_tier_full runtime::tests::agent_claims:: -- --nocapture`
- 阻断条件:
  - 若 runtime 无法保证同一 agent 的单 owner 原子性，则 claim 功能不得进入实现态。
  - 若 viewer / pure API 无法给出 canonical claim 成本与倒计时，则不得宣称 claim 机制可正式使用。
  - 若经济审计无法覆盖 activation fee、upkeep、refund/slash，则不得合入。
- 说明:
  - 本专题是 gameplay 规则与经济边界，不是现实货币付费系统。
  - v1 默认不拍死绝对价格，只先冻结结构、状态机与不可突破的边界。
  - 当前 claim QA 真值已完成，但仓库内仍存在与本专题无关的 `oasis7_hosted_access` bin 编译问题；本轮已通过 `--lib` 定向运行 claim runtime suite，不把该独立缺陷误判为 agent claim blocker。
