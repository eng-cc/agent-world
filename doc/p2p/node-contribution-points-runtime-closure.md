# Agent World Runtime：节点贡献积分运行时闭环（设计文档）

## 目标
- 在运行时侧形成 Node Contribution Points 的闭环：采样、结算、审计、回放一致。
- 为后续节点收益、治理参数调优、发布门禁提供统一口径。

## 范围

### In Scope
- 运行时采样器（snapshot/storage -> epoch settlement）。
- 多节点闭环验证与 `test_tier_required` 回归口径。
- 与既有 `node-contribution-points` 设计的运行时接线收口。

### Out of Scope
- 新经济模型设计。
- 跨链或外部记账系统接入。

## 接口/数据
- 运行时模块：`crates/agent_world/src/runtime/node_points.rs`
- 运行时接线：`crates/agent_world/src/runtime/mod.rs`
- 节点快照输入：`crates/agent_world_node/src/lib.rs`（`NodeSnapshot`）
- 关联设计：`doc/p2p/node-contribution-points.md`

## 里程碑
- M1：完成运行时采样器接线。
- M2：完成多节点闭环测试。
- M3：完成 required 回归与文档收口。

## 风险
- 采样窗口与结算窗口错配可能导致奖励偏差。
- 快照不一致会影响回放可重复性与审计可解释性。
