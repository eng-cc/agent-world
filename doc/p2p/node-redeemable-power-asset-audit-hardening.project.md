# Agent World Runtime：可兑现节点资产与电力兑换闭环（二期审计与签名加固，项目管理文档）

## 任务拆解
- [x] AHA-0：完成设计文档。
- [x] AHA-1：完成项目管理文档拆解。
- [ ] AHA-2：实现结算签名语义升级（`mintsig:v1`）与签名校验函数。
- [ ] AHA-3：实现资产不变量审计报告 API，并覆盖 `test_tier_required` 单测。
- [ ] AHA-4：`world_viewer_live` reward runtime 接入审计摘要输出。
- [ ] AHA-5：文档状态回写、devlog 收口、发布说明整理。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/p2p/node-redeemable-power-asset-audit-hardening.md`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/reward_asset.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/world/resources.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/tests/reward_asset.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/devlog/2026-02-16.md`

## 状态
- 当前阶段：AHA-1 完成（项目拆解已完成）。
- 下一步：执行 AHA-2（结算签名语义升级与签名校验函数）。
- 最近更新：2026-02-16。
