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
  - runtime 已支持 `governance.finality.v1` 与 8 个 controller slot 的 world-state registry，chain runtime 也已支持启动时优先读取 world registry，但目标 execution world 仍需 operator 实际导入并切换，不等于 rotation / revocation / ceremony / QA gate 全部通过
  - finality signer 的 production signing material 仍由人工离线 custody 持有；runtime 不再把 local seed 视为 registry 存在时的真值，但真实外部签名轮换与失效恢复演练仍未执行
  - controller signer policy 虽已支持由 execution world 注入 `NodeRuntime`，但真实 governance account / recipient binding、genesis ceremony 和最终 QA `pass` 仍未完成

## Transition Freeze Snapshot（public-only）
- batch id: `oasis7-governance-batch-20260323-01`
- producer decision: all governance/controller slots default to `threshold_ed25519 2-of-3`
- current finality signer freeze:
  - `governance.finality.v1`
  - allowed_public_keys:
    - `54e7a02919fff2d49a9c325def8cb0211ea7f7a75a9011b9d0678b9e2a7af6bc`
    - `38dac17ff403cc19de033e47be7cf7b5354635fbc5c1976d7c532e20494aace4`
    - `e22bd5029176296712fb1a477f91c15775e5ab858181cb4172839ced526f12c8`
- current controller signer freeze:
  - `msig.genesis.v1` 与 7 个 treasury/controller slot 的 public-only signer set 见 `doc/p2p/token/mainchain-token-genesis-parameter-freeze-sheet-2026-03-22.md` §3A
- note:
  - 上述信息只构成 transition ceremony snapshot，不构成最终 on-chain/world-state registry 完成态

## Execution Workstream Snapshot（2026-03-23）
- [x] 在 `WorldState` 持久化 `governance_finality_signer_registry` 与 `governance_main_token_controller_registry`
- [x] `governance_effective_finality_epoch_snapshot` 在 registry 存在时优先使用 world-state signer truth，而不是 deterministic local seed fallback
- [x] chain runtime 启动时可从 execution world 读取 controller signer policy，并覆盖 `NodeConfig.main_token_controller_binding`
- [x] 新增 `oasis7_governance_registry_import`，可把 operator-local `public_manifest.json` 导入 execution world
- [x] 已用真实 `public_manifest.json` 在临时 world 目录完成 smoke import，验证 3 个 finality signer + 8 个 controller slot 可落入 world-state registry
- [ ] 目标 world 目录 `output/chain-runtime/viewer-live-node/reward-runtime-execution-world` 仍需 operator 执行正式导入
- [ ] rotation / revocation / failover 的真实 runbook 演练与 QA 证据仍待执行
- [ ] genesis address binding / ceremony / QA pass 仍待后续 `MAINNET-3` 收口

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
- `env -u RUSTC_WRAPPER cargo test -p oasis7 governance_finality_registry_roundtrip_persists_and_drives_epoch_snapshot -- --nocapture`
- `env -u RUSTC_WRAPPER cargo test -p oasis7 world_registry_overrides_node_controller_binding -- --nocapture`
- `env -u RUSTC_WRAPPER cargo test -p oasis7 import_writes_governance_registries_into_world -- --nocapture`
- `env -u RUSTC_WRAPPER cargo run -p oasis7 --bin oasis7_governance_registry_import -- --world-dir <target-world-dir> --public-manifest <operator-local-public-manifest.json>`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: completed
- 执行状态: in_progress
- 下一步: 在已有 world-state registry 真值基础上，继续推进 rotation / revocation / failover runbook、把目标节点稳定切到 registry truth 启动路径，并为 `MAINNET-3` 的真实 binding / ceremony / QA pass 准备证据。
- 最近更新: 2026-03-23
