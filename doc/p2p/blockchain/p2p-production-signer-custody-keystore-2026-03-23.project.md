# oasis7 生产级 signer custody / keystore 基线（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] CUSTODY-0 (PRD-P2P-CUSTODY-001/002/003/004) [test_tier_required]: 新建生产级 signer custody / keystore 专题 PRD / design / project，并接入 `doc/p2p` 与 readiness 主追踪。
- [x] CUSTODY-1 (PRD-P2P-CUSTODY-001/002) [test_tier_required]: 盘点 `node runtime signer`、`viewer/player signer`、`governance/controller signer` 当前 bootstrap 真值，冻结环境等级与 blocker。
- [x] CUSTODY-2 (PRD-P2P-CUSTODY-002) [test_tier_required]: 冻结三类 signer 的生产目标后端、禁止项与 source boundary。
- [x] CUSTODY-3 (PRD-P2P-CUSTODY-003) [test_tier_required]: 冻结 rotation、revocation、audit trail 与 operator ownership 为 `MAINNET-1` 的硬门禁。
- [x] CUSTODY-4 (PRD-P2P-CUSTODY-004) [test_tier_required]: 冻结 local/dev/preview/production 环境 policy，并回写 public claims 依赖关系。

## 当前结论
- 当前阶段:
  - 游戏阶段口径: `limited playable technical preview`
  - 安全阶段口径: `crypto-hardened preview`
  - `MAINNET-1`: completed as specification gate
- 选定方案:
  - production signer custody: `offline storage + manual multisig`
  - operator key staging root: operator-local non-repo path（例如 `~/Documents/keys`）
- 当前 blocker:
  - `node.private_key` 仍会自动生成并明文写入 `config.toml`
  - viewer signer 仍可从 env / `config.toml` bootstrap 并注入 HTML
  - governance/controller signer 真值仍未进入正式托管后端

## 依赖
- `crates/oasis7/src/bin/oasis7_chain_runtime/node_keypair_config.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher/viewer_auth_bootstrap.rs`
- `crates/oasis7/src/runtime/world/governance.rs`
- `crates/oasis7_node/src/types.rs`
- `crates/oasis7_node/src/node_runtime_core.rs`
- `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
- `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
- `testing-manual.md`

## 验收命令（本轮）
- `rg -n "config.toml|bootstrap|deterministic local seed|preview-only|managed keystore|external signer" doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.design.md doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.project.md doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md doc/p2p/project.md`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: completed
- 下一步: 进入 execution workstream，把 `offline storage + manual multisig` 方案拆成真实 key generation / storage / signing runbook；operator-local key staging root（例如 `~/Documents/keys`）仅作为人工 custody 的非仓库 staging 根目录，不作为应用自动 keystore。
- 最近更新: 2026-03-23
