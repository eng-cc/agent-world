# Agent World 主链 Token 分配与发行机制（项目管理文档）

## 任务拆解
- [x] TAM-0：完成设计文档与项目管理文档建档。
- [x] TAM-1：实现主链 Token 状态模型、快照字段与基础查询接口。
- [x] TAM-2：实现创世分配初始化与 vesting 领取动作闭环。
- [x] TAM-3：实现 epoch 增发公式与分配执行（含余数确定性策略）。
- [x] TAM-4：实现 Gas/罚没/模块费用销毁与协议金库记账。
- [ ] TAM-5：实现参数治理边界（范围校验、生效延迟、提案审计事件）。
- [ ] TAM-6：实现 NodePoints -> 主链 Token 桥接占位接口并接入结算路径。
- [ ] TAM-7：补齐 `test_tier_required` / `test_tier_full` 测试矩阵与回归脚本。
- [ ] TAM-8：文档回写、发布说明与运行手册补充。

## 依赖
- `world-rule.md`
- `doc/p2p/node-redeemable-power-asset.md`
- `doc/p2p/node-reward-settlement-native-transaction.md`
- `crates/agent_world/src/runtime/reward_asset.rs`
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/runtime/world/resources.rs`
- `testing-manual.md`

## 状态
- 当前阶段：TAM-0 ~ TAM-4 已完成。
- 下一步：TAM-5（参数治理边界：范围校验、生效延迟、提案审计事件）。
- 最近更新：2026-02-26。
