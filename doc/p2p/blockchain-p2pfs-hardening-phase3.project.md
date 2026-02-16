# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 3）项目管理文档

## 任务拆解
- [x] HP3-0：输出设计文档与项目管理文档。
- [x] HP3-1：实现 `ActionEnvelope/WorldHeadAnnounce` 的 ed25519 签名器与验签工具。
- [x] HP3-2：接线 `SequencerMainloop` 双栈签名治理（ed25519 + HMAC 兼容）。
- [ ] HP3-3：补齐测试并执行 `test_tier_required` 回归，回写文档状态与 devlog。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/signature.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/sequencer_mainloop.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/Cargo.toml`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：HP3-0 ~ HP3-2 完成，HP3-3 进行中。
- 阻塞项：无。
- 最近更新：2026-02-17。
