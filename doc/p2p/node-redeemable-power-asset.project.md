# Agent World Runtime：可兑现节点资产与电力兑换闭环（项目管理文档）

## 任务拆解
- [x] RPA-0：完成设计文档与项目管理文档。
- [x] RPA-1：实现 `PowerCredit` 资产账本与配置（含快照持久化）。
- [x] RPA-2：将 `NodePoints` epoch 结算结果接入链状态铸造记录（`NodeRewardMintRecord`）。
- [x] RPA-3：实现 `RedeemPower` 动作闭环（余额扣减、Agent 电力增加、事件产出）。
- [x] RPA-4：实现守恒与风控（储备池、每 epoch 额度、最小兑换单位、nonce 防重放）。
- [x] RPA-5：接线运行时主链路（`world_viewer_live`/runtime 开关与配置）。
- [x] RPA-6：实现最小需求侧支付入口（系统订单池预算分配）并接入结算。
- [ ] RPA-7：实现身份签名治理最小收口（`node_id <-> public_key` 校验，拒绝未绑定提交）。
- [ ] RPA-8：增强 DistFS 证明语义字段并补齐 `test_tier_required`/`test_tier_full` 回归。
- [ ] RPA-9：文档状态回写、devlog 收口、发布说明整理。

## 依赖
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/p2p/node-redeemable-power-asset.md`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/node_points.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/node_points_runtime.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/state.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/runtime/events.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_proto/src/distributed.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_consensus/src/pos.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world/src/bin/world_viewer_live.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/crates/agent_world_distfs/src/lib.rs`
- `/Users/scc/.codex/worktrees/ee97/agent-world/doc/devlog/2026-02-16.md`

## 状态
- 当前阶段：RPA-6 完成（系统订单池预算分配已接入结算）。
- 下一步：执行 RPA-7（身份签名治理最小收口）。
- 最近更新：2026-02-16。
