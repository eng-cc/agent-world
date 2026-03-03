# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 3）项目管理文档（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] HP3-0 (PRD-P2P-MIG-047)：输出设计文档与项目管理文档。
- [x] HP3-1 (PRD-P2P-MIG-047)：实现 `ActionEnvelope/WorldHeadAnnounce` 的 ed25519 签名器与验签工具。
- [x] HP3-2 (PRD-P2P-MIG-047)：接线 `SequencerMainloop` 双栈签名治理（ed25519 + HMAC 兼容）。
- [x] HP3-3 (PRD-P2P-MIG-047)：补齐测试并执行 `test_tier_required` 回归，回写文档状态与 devlog。

## 依赖
- `crates/agent_world_consensus/src/signature.rs`
- `crates/agent_world_consensus/src/sequencer_mainloop.rs`
- `crates/agent_world_consensus/src/lib.rs`
- `crates/agent_world_consensus/Cargo.toml`
- `doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：HP3-0 ~ HP3-3 全部完成。
- 阻塞项：无。
- 最近更新：2026-02-17。
