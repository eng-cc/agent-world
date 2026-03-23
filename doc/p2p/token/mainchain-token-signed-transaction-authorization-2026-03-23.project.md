# oasis7 主链 Token 签名交易鉴权（项目管理文档）

- 对应设计文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.design.md`
- 对应需求文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`

审计轮次: 2
## 任务拆解（含 PRD-ID 映射）
- [x] STRAUTH-0 (PRD-P2P-TXAUTH-001/002/003) [test_tier_required]: 新建“主链 Token 签名交易鉴权”专题 PRD / design / project，并接入 `doc/p2p` 模块主追踪。
- [x] STRAUTH-1 (PRD-P2P-TXAUTH-001/002) [test_tier_required]: 由 `runtime_engineer` 为 `POST /v1/chain/transfer/submit` 实现 `public_key + signature` 鉴权、`awt:pk:` 账户绑定、控制面请求结构同步与定向回归。
- [ ] STRAUTH-2 (PRD-P2P-TXAUTH-001/003) [test_tier_required]: 由 `runtime_engineer` 继续把 `ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury` 纳入统一 signed transaction envelope。
  - [x] STRAUTH-2A [test_tier_required]: 为 `ConsensusActionPayloadEnvelope` 增加主链 Token auth proof，并让 `NodeRuntime` 对 transfer/claim/genesis/treasury 在提交层强制验签。
  - [ ] STRAUTH-2B [test_tier_required]: 将 genesis/treasury 的 `controller_account_id` 与真实治理 signer allowlist / slot binding / ceremony 收口，避免停留在 signed metadata 阶段。
- [ ] STRAUTH-3 (PRD-P2P-TXAUTH-002/003) [test_tier_required + test_tier_full]: 由 `viewer_engineer` + `qa_engineer` 补齐 Web/native 转账签名提交流程、失败提示与更完整回归证据。

## 当前切片结论
- 已收口:
  - `TransferMainToken` 公开 HTTP submit 不再接受未签名请求。
  - `from_account_id` 已绑定到 `awt:pk:<public_key_hex>`。
  - `oasis7_web_launcher` 代理请求结构已同步到新字段集合。
  - `ConsensusActionPayloadEnvelope` 已支持 shared main-token auth proof，`NodeRuntime` 提交层已对 transfer/claim/genesis/treasury 统一做 signed payload gating。
- 仍待完成:
  - genesis/treasury 仍缺真实治理 `controller_account_id` allowlist / slot binding / ceremony 收口，当前只做到 signed metadata。
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
- 下一步: 进入 `STRAUTH-2B`，把 genesis/treasury 的治理 controller binding 从“已签名 metadata”提升到真实 allowlist / slot / ceremony 收口；随后由 `viewer_engineer` 补 `STRAUTH-3` 的签名 UI/交互闭环。
- 最近更新: 2026-03-23
