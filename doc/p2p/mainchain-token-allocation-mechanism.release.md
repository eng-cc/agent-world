# Agent World Runtime：主链 Token 分配与发行机制（发布说明）

## 发布范围
- 设计/项目文档：
  - `doc/p2p/mainchain-token-allocation-mechanism.md`
  - `doc/p2p/mainchain-token-allocation-mechanism.project.md`
- 代码主线：
  - `crates/agent_world/src/runtime/main_token.rs`
  - `crates/agent_world/src/runtime/events.rs`
  - `crates/agent_world/src/runtime/state.rs`
  - `crates/agent_world/src/runtime/state/apply_domain_event_core.rs`
  - `crates/agent_world/src/runtime/world/event_processing.rs`
  - `crates/agent_world/src/runtime/world/event_processing/action_to_event_core.rs`
  - `crates/agent_world/src/runtime/world/resources.rs`
  - `crates/agent_world/src/runtime/tests/main_token.rs`
  - `crates/agent_world/src/runtime/tests/reward_asset_settlement_action.rs`
- 测试脚本与手册：
  - `scripts/main-token-regression.sh`
  - `testing-manual.md`

## 交付摘要
- TAM-1：主链 Token 状态模型、快照字段与查询接口落地。
- TAM-2：创世分配初始化与 vesting 领取动作闭环落地。
- TAM-3：epoch 增发公式、分配与余数确定性策略落地。
- TAM-4：Gas/Slash/Module fee 销毁与 treasury 入账落地。
- TAM-5：参数治理边界、延迟生效与审计事件落地。
- TAM-6：NodePoints -> 主链 Token 桥接占位与结算主路径接线落地。
- TAM-7：`test_tier_required/test_tier_full` 回归矩阵脚本落地。
- TAM-8：文档回写、发布说明与运行手册补充完成。

## 核心接口变化
- Runtime 新增/增强动作：
  - `Action::InitializeMainTokenGenesis`
  - `Action::ClaimMainTokenVesting`
  - `Action::ApplyMainTokenEpochIssuance`
  - `Action::SettleMainTokenFee`
  - `Action::UpdateMainTokenPolicy`
- Runtime 新增/增强事件：
  - `DomainEvent::MainTokenGenesisInitialized`
  - `DomainEvent::MainTokenVestingClaimed`
  - `DomainEvent::MainTokenEpochIssued`
  - `DomainEvent::MainTokenFeeSettled`
  - `DomainEvent::MainTokenPolicyUpdateScheduled`
  - `DomainEvent::NodePointsSettlementApplied`（新增主链 Token 桥接审计字段）
- 状态新增：
  - `main_token_scheduled_policy_updates`
  - `main_token_node_points_bridge_records`
- 查询接口新增（节选）：
  - `main_token_scheduled_policy_update(effective_epoch)`
  - `main_token_node_points_bridge_record(epoch_index)`

## 兼容性说明
- 主链 Token 数值类型统一为 `u64`，并保持快照 `serde(default)` 兼容旧状态恢复。
- 参数治理更新采用固定延迟生效（`2` epoch），保证回放一致与审计可追踪。
- `NodePoints` 与主链 Token 仍是两套账本：前者不是 Token 本体，仅作为桥接分配输入。
- 当前桥接账户映射为占位策略 `account_id = node_id`，后续可升级为独立地址映射层。

## 回归验证
- 主链 Token / NodePoints 桥接定向回归：
```bash
./scripts/main-token-regression.sh required
./scripts/main-token-regression.sh full
```

## 运行手册要点
- 发行配置与参数边界：参考 `doc/p2p/mainchain-token-allocation-mechanism.md`。
- 系统测试矩阵与分层执行：参考 `testing-manual.md` S3 “主链 Token / NodePoints 桥接定向回归（required/full）”。
- 审计排查优先读取：
  - `main_token_epoch_issuance_record(epoch)`
  - `main_token_scheduled_policy_update(effective_epoch)`
  - `main_token_node_points_bridge_record(epoch)`

## 已知事项
- NodePoints 桥接预算仅消费同 epoch 的 `node_service_reward_amount`，不支持任意 treasury 抽取。
- 若该 epoch 未产生主链增发记录，桥接预算为 `0`，不会自动发放主链 Token。
