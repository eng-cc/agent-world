# Agent World Runtime：共识数值语义与原子状态转移硬化（15 点清单第一阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase1.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase1.project.md`

### T1 数值语义（显式溢出错误）
- [ ] `ResourceStock::add` 改为 `checked_add` 并增加 `StockError::Overflow`
- [ ] `ResourceDelta::add_assign` 改为可失败聚合，`merge_rule_decisions` 透传 `CostOverflow`
- [ ] `World::adjust_resource_balance` / `apply_resource_delta` 改为可失败接口

### T2 状态转移原子性
- [ ] `ResourceTransferred`：先校验余额与目标上界，再统一提交
- [ ] `PowerRedeemed`：先校验目标资源上界与储备约束，再执行扣减与写入
- [ ] `EconomicContractSettled`：先校验借贷双方资源，再统一提交

### T3 执行流接线
- [ ] `step_with_modules` 处理资源 delta 溢出为 `ActionRejected`，不提交部分状态
- [ ] 错误信息收敛为可观测原因（便于后续告警）

### T4 回归与收口
- [ ] 补充溢出拒绝与原子性测试
- [ ] 运行对应 `test_tier_required` 口径测试
- [ ] 更新设计/项目文档状态与 `doc/devlog/2026-02-22.md`

## 依赖
- Runtime 核心：
  - `crates/agent_world/src/runtime/world/step.rs`
  - `crates/agent_world/src/runtime/world/resources.rs`
  - `crates/agent_world/src/runtime/rules.rs`
  - `crates/agent_world/src/runtime/state/apply_domain_event_core.rs`
  - `crates/agent_world/src/runtime/state/apply_domain_event_gameplay.rs`
- 资源模型：
  - `crates/agent_world/src/simulator/types.rs`
- 测试：
  - `crates/agent_world/src/runtime/tests/basic.rs`
  - `crates/agent_world/src/runtime/tests/gameplay_protocol.rs`
  - `crates/agent_world/src/simulator/tests/basics.rs`

## 状态
- 当前状态：`T1 进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3、T4
- 阻塞项：无
