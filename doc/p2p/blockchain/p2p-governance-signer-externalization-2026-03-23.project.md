# oasis7 治理 signer 外部化与轮换门禁（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] GOVSIGN-0 (PRD-P2P-GOVSIGN-001/002/003/004) [test_tier_required]: 新建治理 signer 外部化专题 PRD / design / project，并接入 `doc/p2p` 与 readiness 主追踪。
- [x] GOVSIGN-1 (PRD-P2P-GOVSIGN-001/002) [test_tier_required]: 盘点 finality/controller signer 当前 local seed/config 真值，冻结环境等级与 blocker。
- [x] GOVSIGN-2 (PRD-P2P-GOVSIGN-002) [test_tier_required]: 冻结两类治理 signer 的长期 source-of-truth、update authority 与禁止项。
- [x] GOVSIGN-3 (PRD-P2P-GOVSIGN-003) [test_tier_required]: 冻结 failover、rotation、revocation 与 operator ownership gate。
- [x] GOVSIGN-4 (PRD-P2P-GOVSIGN-004) [test_tier_required]: 冻结 readiness/public-claims/ceremony 对 governance signer 的前置依赖。

## 当前结论
- 当前阶段:
  - 游戏阶段口径: `limited playable technical preview`
  - 安全阶段口径: `crypto-hardened preview`
  - `MAINNET-2`: completed as specification gate
- 选定方案:
  - governance truth target: `on-chain/world-state registry`
- 当前 blocker:
  - governance finality signer 仍由 deterministic local seed 派生
  - controller signer policy 真值仍由 `NodeConfig` 本地配置承担

## 依赖
- `crates/oasis7/src/runtime/world/governance.rs`
- `crates/oasis7_node/src/types.rs`
- `crates/oasis7_node/src/node_runtime_core.rs`
- `crates/oasis7/src/consensus_action_payload.rs`
- `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`
- `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
- `testing-manual.md`

## 验收命令（本轮）
- `rg -n "deterministic local seed|controller_signer_policies|NodeConfig|externalized|failover|revocation" doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.design.md doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.project.md doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md doc/p2p/project.md`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: completed
- 下一步: 进入 execution workstream，把 governance signer long-term truth 直接迁入链上/world-state registry；链下 registry 只允许做临时迁移工具，不得作为最终完成态。
- 最近更新: 2026-03-23
