# oasis7 主链 Token 初始分配与早期贡献奖励口径（项目管理文档）

- 对应设计文档: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.design.md`
- 对应需求文档: `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.prd.md`

审计轮次: 2
## 任务拆解（含 PRD-ID 映射）
- [x] TIGR-0 (PRD-P2P-TOKEN-INIT-001/002/003) [test_tier_required]: 完成 Token 初始分配与早期贡献奖励专题 PRD / design / project 建档，并接入 `doc/p2p` 模块主追踪。
- [x] TIGR-1 (PRD-P2P-TOKEN-INIT-001/002) [test_tier_required]: 由 `runtime_engineer` 输出创世 bucket/account/recipient/vesting 参数表草案，明确当前实现下所有创世 bucket 都先进入 recipient `vested_balance`，并区分 custody account 与 post-genesis treasury bucket 语义。
- [x] TIGR-2 (PRD-P2P-TOKEN-INIT-002/003) [test_tier_required]: 由 `qa_engineer` 建立创世配置审计清单，覆盖 `sum=10000 bps`、单人直持上限、创世液态流通上限、首年外部释放上限与 custody/treasury 语义边界。
- [ ] TIGR-3 (PRD-P2P-TOKEN-INIT-003) [test_tier_required]: 由 `liveops_community` 输出 limited preview 早期贡献奖励评分模板、证据字段与对外禁语清单。
- [ ] TIGR-4 (PRD-P2P-TOKEN-INIT-002/003) [test_tier_required]: 由 `producer_system_designer` 基于 `TIGR-1~3` 做最终发行前评审，决定 early contributor reserve 是保持多签治理执行还是后续合并进 `ecosystem_pool` 路径。

## TIGR-1 产物（本地草案，待 review）
| bucket_id | ratio_bps | recipient | start_epoch | cliff_epochs | linear_unlock_epochs | genesis_liquid | ownership_note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `team_long_term_vesting` | `2000` | `protocol:team-core-vesting` | `0` | `365` | `1095` | `0` | 团队多签/受益人表内部分配，个人份额不得突破 `1500 bps` 上限 |
| `early_contributor_reward_reserve` | `1500` | `protocol:early-contributor-reward` | `0` | `0` | `3650` | `0` | 贡献奖励多签储备，不等于 marketing airdrop 池 |
| `node_service_genesis_custody` | `2000` | `protocol:node-service-custody` | `0` | `180` | `1825` | `0` | 创世 custody，后续是否并入/补充 treasury 需单独决议 |
| `staking_genesis_custody` | `1500` | `protocol:staking-custody` | `0` | `180` | `1825` | `0` | 创世 custody，和 runtime `staking_reward_pool` 分开 |
| `ecosystem_governance_reserve` | `1500` | `protocol:ecosystem-governance` | `0` | `90` | `1460` | `0` | 生态治理储备 |
| `security_reserve_emergency` | `1000` | `protocol:security-council-reserve` | `0` | `0` | `0` | `0` | 安全委员会应急盘 |
| `foundation_ops_reserve` | `500` | `protocol:foundation-ops` | `0` | `90` | `730` | `0` | 运营与基础设施盘 |

## TIGR-1 验证
- `ratio_bps` 总和为 `10000`
- 项目战略控制总和为 `5000 bps`
- 协议长期储备总和为 `3500 bps`
- 全部 bucket `genesis_liquid=0`
- `recipient` 当前均为 custody account 命名草案，不假装已初始化 treasury bucket

## 依赖
- `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md`
- `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`
- `doc/testing/governance/token-genesis-allocation-audit-checklist-2026-03-22.prd.md`
- `doc/testing/evidence/token-genesis-allocation-audit-template-2026-03-22.md`
- `doc/p2p/prd.md`
- `doc/game/prd.md`
- `doc/game/gameplay/gameplay-limited-preview-execution-2026-03-22.prd.md`
- `crates/oasis7/src/runtime/main_token.rs`
- `testing-manual.md`

## 状态
- 当前阶段：active
- 下一步：执行 `TIGR-3`，由 `liveops_community` 基于当前 `TIGR-1/TIGR-2` 输出 limited preview 早期贡献奖励评分模板、证据字段与禁语清单。
- 最近更新：2026-03-22
- 备注：`TIGR-1/TIGR-2` 已落盘，但仍未执行真实创世或真实对外发币；在 `TIGR-3/TIGR-4` 完成前，仍不得把早期贡献奖励写成公开发币活动。
