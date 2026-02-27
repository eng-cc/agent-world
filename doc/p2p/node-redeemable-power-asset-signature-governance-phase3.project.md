# Agent World Runtime：可兑现节点资产与电力兑换闭环（三期真实签名与治理闭环，项目管理文档）

## 任务拆解
- [x] SGC-0：完成三期设计文档。
- [x] SGC-1：完成三期项目管理文档拆解。
- [x] SGC-2：落地 `mintsig:v2`（ed25519）签名/验签与治理策略结构。
- [x] SGC-3：实现签名版兑换动作与治理门禁（策略要求时拒绝无签名兑换）。
- [x] SGC-4：接线 `world_viewer_live` reward runtime（真实私钥结算 + 签名兑换）。
- [x] SGC-5：补齐 `test_tier_required` 回归、回写文档状态与 devlog 收口。

## 依赖
- `doc/p2p/node-redeemable-power-asset-signature-governance-phase3.md`
- `crates/agent_world/src/runtime/reward_asset.rs`
- `crates/agent_world/src/runtime/world/resources.rs`
- `crates/agent_world/src/runtime/world/event_processing.rs`
- `crates/agent_world/src/runtime/events.rs`
- `crates/agent_world/src/runtime/tests/reward_asset.rs`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `doc/devlog/2026-02-17.md`

## 状态
- 当前阶段：SGC-0 ~ SGC-5 全部完成。
- 阻塞项：无。
- 最近更新：2026-02-17。
