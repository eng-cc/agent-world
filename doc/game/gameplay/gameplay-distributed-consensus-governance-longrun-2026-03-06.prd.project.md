# Gameplay Distributed Consensus Governance Long-Run（项目管理文档）

审计轮次: 3

## 审计备注
- 主项目入口：`doc/game/gameplay/gameplay-top-level-design.prd.project.md`
- 专题 PRD：`doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
- 本文档仅维护“分布式共识 + 治理 + 反女巫”专题任务拆解与执行状态。

## 任务拆解（含 PRD-ID 映射）

### T0 文档设计建档
- [x] TASK-GAME-DCG-000 (PRD-GAME-005): 新增专题 PRD 与项目管理文档，完成文档树挂载。

### T1 执行共识层（RSM）
- [x] TASK-GAME-DCG-001 (PRD-GAME-005-01) [test_tier_required]: 新增 `TickBlock/TickCertificate` 数据结构与持久化写入。
- [x] TASK-GAME-DCG-002 (PRD-GAME-005-01) [test_tier_required]: 补齐 `events_hash/state_root` 计算与多次回放一致性回归（full-tier 长稳覆盖并入 `TASK-GAME-DCG-009`）。

### T2 治理共识层（规则与参数）
- [ ] TASK-GAME-DCG-003 (PRD-GAME-005-02) [test_tier_required]: 将规则/参数变更收敛到治理事件流，禁止旁路应用。
- [ ] TASK-GAME-DCG-004 (PRD-GAME-005-02) [test_tier_required]: 落地 `timelock + epoch` 生效门禁，补齐提前 apply 拒绝测试。

### T3 紧急控制与宪章
- [ ] TASK-GAME-DCG-005 (PRD-GAME-005-02) [test_tier_required]: 新增紧急刹车/紧急否决状态机与阈值校验。
- [ ] TASK-GAME-DCG-006 (PRD-GAME-005-02) [test_tier_required]: 宪章化权限与审计字段，补齐越权拒绝回归。

### T4 身份与反女巫
- [ ] TASK-GAME-DCG-007 (PRD-GAME-005-03) [test_tier_required]: 新增身份信誉+抵押权重快照模型。
- [ ] TASK-GAME-DCG-008 (PRD-GAME-005-03) [test_tier_required + test_tier_full]: 女巫攻击模拟、惩罚与申诉闭环测试。

### T5 长稳与发布门禁
- [ ] TASK-GAME-DCG-009 (PRD-GAME-005-01/002/003) [test_tier_full]: 7~30 天 soak + 故障注入，验证恢复与证书链一致性。
- [ ] TASK-GAME-DCG-010 (PRD-GAME-005-01/002/003) [test_tier_required]: 输出发布门禁报告与回滚预案。

## 依赖
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-engineering-architecture.md`
- `doc/game/gameplay/gameplay-runtime-governance-closure.prd.md`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-06
- 当前状态: `active`
- 下一任务: `TASK-GAME-DCG-003`（治理变更收敛到事件流并禁止旁路）
- ROUND-002 进展: `TASK-GAME-DCG-001/002` 已完成，tick 证书链在 `step/replay/save-load` 闭环可用。
- 阻塞项: 无
