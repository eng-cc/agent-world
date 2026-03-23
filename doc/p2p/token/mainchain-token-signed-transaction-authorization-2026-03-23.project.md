# oasis7 主链 Token 签名交易鉴权（项目管理文档）

- 对应设计文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.design.md`
- 对应需求文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`

审计轮次: 2
## 任务拆解（含 PRD-ID 映射）
- [x] STRAUTH-0 (PRD-P2P-TXAUTH-001/002/003) [test_tier_required]: 新建“主链 Token 签名交易鉴权”专题 PRD / design / project，并接入 `doc/p2p` 模块主追踪。
- [x] STRAUTH-1 (PRD-P2P-TXAUTH-001/002) [test_tier_required]: 由 `runtime_engineer` 为 `POST /v1/chain/transfer/submit` 实现 `public_key + signature` 鉴权、`awt:pk:` 账户绑定、控制面请求结构同步与定向回归。
- [x] STRAUTH-2 (PRD-P2P-TXAUTH-001/003) [test_tier_required]: 由 `runtime_engineer` 继续把 `ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury` 纳入统一 signed transaction envelope。
  - [x] STRAUTH-2A [test_tier_required]: 为 `ConsensusActionPayloadEnvelope` 增加主链 Token auth proof，并让 `NodeRuntime` 对 transfer/claim/genesis/treasury 在提交层强制验签。
  - [x] STRAUTH-2B [test_tier_required]: 将 genesis/treasury 的治理控制从“signed metadata”推进到正式 controller slot binding，并继续保留 signer allowlist / ceremony 后续任务。
    - [x] STRAUTH-2B1 [test_tier_required]: 为 genesis/treasury 建立正式 controller slot registry，并在 `NodeRuntime` 提交层按 `action/bucket` 绑定 `auth.account_id`。
    - [x] STRAUTH-2B2 [test_tier_required]: 将 controller slot 继续收口到本地配置 signer allowlist / threshold enforcement，并明确 ceremony / external signer 仍待后续专题。
- [ ] STRAUTH-3 (PRD-P2P-TXAUTH-002/003) [test_tier_required + test_tier_full]: 由 `viewer_engineer` + `qa_engineer` 补齐 Web/native 转账签名提交流程、失败提示与更完整回归证据。

## 当前切片结论
- 已收口:
  - `TransferMainToken` 公开 HTTP submit 不再接受未签名请求。
  - `from_account_id` 已绑定到 `awt:pk:<public_key_hex>`。
  - `oasis7_web_launcher` 代理请求结构已同步到新字段集合。
  - `ConsensusActionPayloadEnvelope` 已支持 shared main-token auth proof，`NodeRuntime` 提交层已对 transfer/claim/genesis/treasury 统一做 signed payload gating。
  - `InitializeMainTokenGenesis / DistributeMainTokenTreasury` 已进入正式 controller slot registry，submit-layer 不再接受任意 controller label。
  - `InitializeMainTokenGenesis / DistributeMainTokenTreasury` 已进入代码级 controller signer allowlist / threshold enforcement，submit-layer 会拒绝 policy missing、allowlist miss 与 threshold 不达标的 proof。
- 仍待完成:
  - genesis/treasury 仍缺 ceremony freeze、external signer、HSM/KMS 与更长期的 world-state / governance source of truth。
  - Web/native UI 仍未真正产出签名材料的交互闭环。
  - 生产级 keystore / signer rotation / external signer 专题。

## 依赖
- `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`
- `doc/p2p/prd.md`
- `doc/p2p/project.md`
- `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api.rs`
- `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api_tests.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher/control_plane.rs`
- `crates/oasis7/src/consensus_action_payload.rs`
- `crates/oasis7/src/runtime/main_token.rs`
- `crates/oasis7_node/src/node_runtime_core.rs`
- `crates/oasis7_node/src/tests_action_payload.rs`
- `testing-manual.md`

## 验收命令（本轮）
- `env -u RUSTC_WRAPPER cargo test -p oasis7 transfer_submit --bin oasis7_chain_runtime`
- `env -u RUSTC_WRAPPER cargo test -p oasis7 submit_chain_transfer_remote --bin oasis7_web_launcher`
- `env -u RUSTC_WRAPPER cargo test -p oasis7_node action_payload`
- `env -u RUSTC_WRAPPER cargo check -p oasis7 --bin oasis7_chain_runtime --bin oasis7_web_launcher`
- `env -u RUSTC_WRAPPER cargo check -p oasis7_node`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前阶段: active
- 下一步: 进入 `STRAUTH-3`，由 `viewer_engineer` + `qa_engineer` 补签名 UI/交互闭环与更完整回归；并另开专题把 ceremony / external signer / keystore 收成长期治理真值。
- 最近更新: 2026-03-23
