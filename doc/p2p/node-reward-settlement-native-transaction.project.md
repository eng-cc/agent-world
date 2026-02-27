# Agent World Runtime：奖励结算切换到网络共识主路径原生交易（项目管理文档）

## 任务拆解
- [x] NSTX-0：完成设计文档。
- [x] NSTX-1：完成项目管理文档拆解。
- [x] NSTX-2：新增结算原生交易 Action/DomainEvent，并接入状态应用与预算扣减语义。
- [x] NSTX-3：在 `event_processing` 增加结算交易签名/守恒/预算校验。
- [x] NSTX-4：切换 `world_viewer_live` reward runtime 到原生交易路径并补测试。
- [x] NSTX-5：执行 `test_tier_required` 回归，回写文档状态与 devlog 收口。

## 依赖
- `doc/p2p/node-reward-settlement-native-transaction.md`
- `crates/agent_world/src/runtime/events.rs`
- `crates/agent_world/src/runtime/world/event_processing.rs`
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `crates/agent_world/src/runtime/tests/reward_asset.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests.rs`

## 状态
- 当前阶段：NSTX-0~NSTX-5 全部完成；奖励结算已切换为网络共识主路径原生交易。
- 阻塞项：无。
- 最近更新：2026-02-17。
