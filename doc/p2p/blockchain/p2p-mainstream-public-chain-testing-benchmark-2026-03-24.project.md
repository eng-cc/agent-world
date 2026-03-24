# oasis7 主流公链测试体系对标与缺口矩阵（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-mainstream-public-chain-testing-benchmark-2026-03-24.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-mainstream-public-chain-testing-benchmark-2026-03-24.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] BENCH-0 (PRD-P2P-BENCH-001/002/003/004) [test_tier_required]: 新建 benchmark 专题 PRD / design / project，并接入 `doc/p2p` 模块主追踪。
- [x] BENCH-1 (PRD-P2P-BENCH-001/002) [test_tier_required]: 冻结主流公链测试分层模型与 oasis7 等价要求。
- [x] BENCH-2 (PRD-P2P-BENCH-002/003) [test_tier_required]: 映射 oasis7 当前 suites/evidence 到 benchmark layers，形成 gap matrix。
- [x] BENCH-3 (PRD-P2P-BENCH-003/004) [test_tier_required]: 冻结 producer 下一步优先级与 public claims 边界。

## 当前结论
- 当前阶段:
  - 游戏阶段口径: `limited playable technical preview`
  - 安全阶段口径: `crypto-hardened preview`
  - 总 verdict: `not_mainnet_grade`
- 当前 benchmark 结论:
  - `L0/L1/L3`: 已有正式基础
  - `L2`: 已有基础，但仍偏库测/长跑，缺 shared network 维度
  - `L4`: 长跑已有，clone-world 与 default/live execution world 的首轮 low-risk governance drill 证据已完成，但覆盖范围仍有限
  - `L5`: shared network / release train 缺失

## 依赖
- `testing-manual.md`
- `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`

## 验收命令（本轮）
- `rg -n "shared network|release train|fuzz/property|governance drill|mainstream public-chain" doc/p2p/blockchain/p2p-mainstream-public-chain-testing-benchmark-2026-03-24.prd.md doc/p2p/blockchain/p2p-mainstream-public-chain-testing-benchmark-2026-03-24.design.md doc/p2p/blockchain/p2p-mainstream-public-chain-testing-benchmark-2026-03-24.project.md doc/p2p/prd.md doc/p2p/project.md testing-manual.md`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: completed
- 下一步: 扩更多 governance slot / finality slot 的真实 drill 覆盖，并定义 `shared network / release train` 的最小执行形态；在此之前，不升级“对标主流公链测试成熟度”相关口径。
- 最近更新: 2026-03-24
