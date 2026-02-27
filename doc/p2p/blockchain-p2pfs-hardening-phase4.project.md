# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 4）项目管理文档

## 任务拆解
- [x] HP4-0：输出设计文档与项目管理文档。
- [x] HP4-1：实现 membership signer 的 ed25519 签发与验签（snapshot/revocation）。
- [x] HP4-2：实现 keyring 双栈 key 管理（HMAC + ed25519）并接入发布/同步路径。
- [x] HP4-3：补齐测试，执行回归，回写文档状态与 devlog。

## 依赖
- `crates/agent_world_consensus/src/membership.rs`
- `crates/agent_world_consensus/src/membership_logic.rs`
- `crates/agent_world_consensus/src/membership_tests.rs`
- `doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：HP4-0 ~ HP4-3 全部完成。
- 阻塞项：无。
- 最近更新：2026-02-17。
