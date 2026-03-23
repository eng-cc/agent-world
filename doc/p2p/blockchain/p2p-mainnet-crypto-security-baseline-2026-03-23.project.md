# oasis7 主链/共识密码学安全基线评估（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] CRYPTO-0 (PRD-P2P-CRYPTO-001/002/003) [test_tier_required]: 新建密码学安全基线评估专题 PRD / design / project，并接入 `doc/p2p` 模块主追踪。
- [x] CRYPTO-1 (PRD-P2P-CRYPTO-001/002) [test_tier_required]: 盘点当前 `viewer`、`node/replication`、`runtime token`、`governance`、`genesis freeze sheet` 的代码真值，形成七个面向的当前状态矩阵。
- [x] CRYPTO-2 (PRD-P2P-CRYPTO-001/002/003) [test_tier_required]: 输出 producer verdict，明确当前整体等级为 `not_mainnet_grade`，并固定系统级 blocker 列表。
- [x] CRYPTO-3 (PRD-P2P-CRYPTO-002/003) [test_tier_required]: 给出 `P0/P1/P2` 路线图、口径边界与下一个必须启动的安全专题。

## 当前 blocker
- `TransferMainToken` 提交入口仍未绑定账户所有权签名。
- 统一的 signed consensus action / asset action 模型仍缺失。
- 节点生产 signer 仍依赖本地明文 `config.toml` keypair。
- 治理 finality signer 仍存在 deterministic local seed 路径。
- 创世 freeze sheet 仍存在 `TBD_BEFORE_MINT` / `pending_binding`。

## 下一专题建议
- `P0` 必开：主链 Token 签名交易鉴权专题。
  - 目标：为 `InitializeMainTokenGenesis`、`ClaimMainTokenVesting`、`TransferMainToken`、`DistributeMainTokenTreasury` 补统一签名交易模型与账户所有权校验。
  - owner role: `producer_system_designer` 牵头，`runtime_engineer` 落实现口径，`qa_engineer` 定最终 block/pass.

## 依赖
- `crates/oasis7/src/viewer/auth.rs`
- `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api.rs`
- `crates/oasis7/src/consensus_action_payload.rs`
- `crates/oasis7/src/runtime/events.rs`
- `crates/oasis7/src/runtime/main_token.rs`
- `crates/oasis7/src/runtime/world/governance.rs`
- `crates/oasis7/src/bin/oasis7_chain_runtime/node_keypair_config.rs`
- `crates/oasis7_node/src/replication.rs`
- `doc/p2p/token/mainchain-token-genesis-parameter-freeze-sheet-2026-03-22.md`
- `testing-manual.md`

## 状态
- 当前阶段: completed
- 下一步: 启动“主链 Token 签名交易鉴权”专题；在该专题收口前，不推进“对标主流公链安全”口径，也不把创世地址 ceremony 当成当前第一优先级。
- 最近更新: 2026-03-23
