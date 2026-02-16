# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 2）项目管理文档

## 任务拆解
- [x] HP2-0：输出设计文档与项目管理文档。
- [ ] HP2-1：Node PoS gossip 接入签名/验签（proposal/attestation/commit）。
- [ ] HP2-2：Node PoS 状态持久化与恢复（重启续跑）。
- [ ] HP2-3：执行回归测试，更新文档状态与 devlog 收口。

## 依赖
- `crates/agent_world_node`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `doc/world-runtime/blockchain-p2pfs-hardening-phase1.md`
- `doc/world-runtime/node-keypair-config-bootstrap.md`

## 状态
- 当前阶段：HP2-0 已完成，进入 HP2-1。
- 下一步：完成 Node PoS gossip proposal/attestation/commit 的签名验签闭环。
- 最近更新：2026-02-16。
