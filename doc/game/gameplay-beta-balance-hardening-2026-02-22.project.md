# Gameplay 内测数值加固（项目管理文档）

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/game/gameplay-beta-balance-hardening-2026-02-22.md`
- [x] 新建项目管理文档：`doc/game/gameplay-beta-balance-hardening-2026-02-22.project.md`

### T1 治理投票权重上限
- [x] 为 `CastGovernanceVote.weight` 增加上限校验
- [x] 补充/更新拒绝路径测试

### T2 策略更新治理授权
- [x] 为 `UpdateGameplayPolicy` 增加治理授权门槛（已通过提案 + 最小总票重）
- [x] 补充授权/未授权测试

### T3 合约声誉去通胀
- [ ] 调整成功结算声誉奖励公式（金额相关 + 质押约束 + 奖励上限）
- [ ] 更新既有数值断言测试

### T4 收口
- [ ] 跑定向测试并记录结果
- [ ] 回写项目状态与文档
- [ ] 更新 `doc/devlog/2026-02-22.md`

## 依赖
- `crates/agent_world/src/runtime/world/event_processing/*`
- `crates/agent_world/src/runtime/tests/gameplay_protocol.rs`
- `doc/game/gameplay-war-politics-mvp-baseline.md`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1、T2
- 进行中：T3
- 未开始：T4
- 阻塞项：无
