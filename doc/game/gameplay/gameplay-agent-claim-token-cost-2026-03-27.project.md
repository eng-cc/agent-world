# Gameplay Agent 认领代币成本与维护机制（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.prd.md`

审计轮次: 1

## 任务拆解

- [x] TASK-GAMEPLAY-AGC-001 (`PRD-GAME-011`) [test_tier_required]: `producer_system_designer` 已建立 agent claim 成本专题，冻结“首个也不免费”的规则边界、三段式成本结构、回收条件与 root 文档挂载。
- [ ] TASK-GAMEPLAY-AGC-002 (`PRD-GAME-011`) [test_tier_required + test_tier_full]: `runtime_engineer` 落地 canonical claim 状态机、main token 扣费/锁定/退款/惩罚记账、epoch upkeep 结算与事件审计。
- [ ] TASK-GAMEPLAY-AGC-003 (`PRD-GAME-011`) [test_tier_required]: `viewer_engineer` 落地未认领报价、已认领状态、cooldown / grace / idle reclaim 倒计时、cap 阻断原因与 pure API 字段对齐。
- [ ] TASK-GAMEPLAY-AGC-004 (`PRD-GAME-011`) [test_tier_required + test_tier_full]: `qa_engineer` 建立 claim 并发、欠费、闲置、cap、refund/slash 与经济审计回归矩阵。
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
- 下一任务: `TASK-GAMEPLAY-AGC-002`
- 已完成补充:
  - `TASK-GAMEPLAY-AGC-001` 已新增 `doc/game/gameplay/gameplay-agent-claim-token-cost-2026-03-27.{prd,design,project}.md`，并将 `PRD-GAME-011` 挂入 game 根 PRD / project / 索引 / README。
- 阻断条件:
  - 若 runtime 无法保证同一 agent 的单 owner 原子性，则 claim 功能不得进入实现态。
  - 若 viewer / pure API 无法给出 canonical claim 成本与倒计时，则不得宣称 claim 机制可正式使用。
  - 若经济审计无法覆盖 activation fee、upkeep、refund/slash，则不得合入。
- 说明:
  - 本专题是 gameplay 规则与经济边界，不是现实货币付费系统。
  - v1 默认不拍死绝对价格，只先冻结结构、状态机与不可突破的边界。
