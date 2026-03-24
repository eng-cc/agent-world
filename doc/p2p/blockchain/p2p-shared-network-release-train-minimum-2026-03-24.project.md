# oasis7 shared network / release train 最小执行形态（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] RTMIN-0 (PRD-P2P-RTMIN-001/002/003/004) [test_tier_required]: 新建 shared network / release train minimum 专题 PRD / design / project，并接入 `doc/p2p` 模块主追踪与 `testing-manual`。
- [ ] RTMIN-1 (PRD-P2P-RTMIN-001/002) [test_tier_required]: `runtime_engineer` 冻结 `release_candidate_bundle` 真值、版本 pinning 与 drift blocker。
- [ ] RTMIN-2 (PRD-P2P-RTMIN-003) [test_tier_required]: `qa_engineer` 冻结 `shared_devnet/staging/canary` 的 `pass/partial/block` 证据模板与 gate 表。
- [ ] RTMIN-3 (PRD-P2P-RTMIN-004) [test_tier_required]: `liveops_community` 冻结 promotion/freeze/rollback/run window/public claims runbook。
- [ ] RTMIN-4 (PRD-P2P-RTMIN-002/003) [test_tier_required + test_tier_full]: 执行 first shared-devnet dry run，落下 candidate/evidence/incident 产物。
- [ ] RTMIN-5 (PRD-P2P-RTMIN-003/004) [test_tier_required + test_tier_full]: 执行 first staging rehearsal 与 first canary rehearsal，并做 freeze/rollback 演练。

## 当前结论
- 当前阶段:
  - 游戏阶段口径: `limited playable technical preview`
  - 安全阶段口径: `crypto-hardened preview`
  - shared network verdict: `specified_not_executed`
- 当前缺口:
  - 没有正式 `shared_devnet/staging/canary`
  - 没有统一 `release_candidate_bundle`
  - 没有 release window / freeze / rollback 正式 runbook

## 依赖
- `testing-manual.md`
- `doc/p2p/blockchain/p2p-mainstream-public-chain-testing-benchmark-2026-03-24.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`

## 验收命令（本轮）
- `rg -n "shared_devnet|staging|canary|release_candidate_bundle|specified_not_executed|release train" doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.prd.md doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.design.md doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.project.md doc/p2p/prd.md doc/p2p/project.md testing-manual.md`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: active
- 下一步: 优先完成 `RTMIN-1~3` 的 candidate bundle、QA gate 与 liveops runbook；在 first shared-devnet dry run 之前，不升级任何 shared-network 或 release-train 对外口径。
- 最近更新: 2026-03-24
