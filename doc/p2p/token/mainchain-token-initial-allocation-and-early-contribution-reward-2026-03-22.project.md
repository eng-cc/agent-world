# oasis7 主链 Token 初始分配与早期贡献奖励口径（项目管理文档）

- 对应设计文档: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.design.md`
- 对应需求文档: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] TIGR-0 (PRD-P2P-TOKEN-INIT-001/002/003) [test_tier_required]: 完成 Token 初始分配与早期贡献奖励专题 PRD / design / project 建档，并接入 `doc/p2p` 模块主追踪。
- [ ] TIGR-1 (PRD-P2P-TOKEN-INIT-001/002) [test_tier_required]: 由 `runtime_engineer` 输出创世 bucket/account/recipient/vesting 参数表，明确哪些走多签、哪些走 treasury、哪些必须保持 `liquid=0`。
- [ ] TIGR-2 (PRD-P2P-TOKEN-INIT-002/003) [test_tier_required]: 由 `qa_engineer` 建立创世配置审计清单，覆盖 `sum=10000 bps`、单人直持上限、创世液态流通上限与首年外部释放上限。
- [ ] TIGR-3 (PRD-P2P-TOKEN-INIT-003) [test_tier_required]: 由 `liveops_community` 输出 limited preview 早期贡献奖励评分模板、证据字段与对外禁语清单。
- [ ] TIGR-4 (PRD-P2P-TOKEN-INIT-002/003) [test_tier_required]: 由 `producer_system_designer` 基于 `TIGR-1~3` 做最终发行前评审，决定 early contributor reserve 是保持多签治理执行还是后续合并进 `ecosystem_pool` 路径。

## 依赖
- `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md`
- `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`
- `doc/p2p/prd.md`
- `doc/game/prd.md`
- `doc/game/gameplay/gameplay-limited-preview-execution-2026-03-22.prd.md`
- `crates/oasis7/src/runtime/main_token.rs`
- `testing-manual.md`

## 状态
- 当前阶段：active
- 下一步：执行 `TIGR-1`，把比例口径映射成具体创世参数表。
- 最近更新：2026-03-22
- 备注：本专题当前只冻结 producer 口径，不等于已经执行真实创世或真实对外发币。
