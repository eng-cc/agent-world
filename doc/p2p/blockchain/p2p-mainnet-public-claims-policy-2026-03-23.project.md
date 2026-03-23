# oasis7 mainnet/public claims policy 复评（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] CLAIMS-0 (PRD-P2P-CLAIMS-001/002/003/004) [test_tier_required]: 新建 mainnet/public claims policy 复评专题 PRD / design / project，并接入 `doc/p2p` 与 readiness 主追踪。
- [x] CLAIMS-1 (PRD-P2P-CLAIMS-001/002) [test_tier_required]: 复核 `MAINNET-1~3` 当前只完成 spec gate 的真值，冻结最终 verdict 与 claim allowlist/denylist。
- [x] CLAIMS-2 (PRD-P2P-CLAIMS-002/003) [test_tier_required]: 冻结 denylist 词汇与 future upgrade condition。
- [x] CLAIMS-3 (PRD-P2P-CLAIMS-003/004) [test_tier_required]: 冻结 execution blocker handoff 与下轮复评触发条件。
- [x] CLAIMS-4 (PRD-P2P-CLAIMS-001/004) [test_tier_required]: 回写 readiness project 与模块主追踪，完成最终 public claims policy 收口。

## 当前结论
- 当前阶段:
  - 游戏阶段口径: `limited playable technical preview`
  - 安全阶段口径: `crypto-hardened preview`
  - 总 verdict: `not_mainnet_grade`
  - `MAINNET-4`: completed
- 当前 blocker:
  - production signer custody 仅完成规格 gate，未完成实装
  - governance signer externalization 仅完成规格 gate，未完成实装
  - genesis freeze/ceremony/QA 仅完成规格 gate，未完成真实绑定与最终 `pass`

## 依赖
- `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-genesis-freeze-ceremony-qa-gate-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`
- `testing-manual.md`

## 验收命令（本轮）
- `rg -n "not_mainnet_grade|crypto-hardened preview|mainnet-grade|production mint ready|spec gate|execution" doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.design.md doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.project.md doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.project.md doc/p2p/project.md`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: completed
- 下一步: 若要继续推进，只能进入 execution workstreams，分别落地 signer custody、governance truth externalization、genesis binding/ceremony/QA；在这些项完成前，继续执行当前 public claims policy。
- 最近更新: 2026-03-23
