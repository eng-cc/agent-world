# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位拉取与 outcome 汇总（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现严格时间水位增量拉取入口
- [x] 实现 outcome 聚合汇总入口
- [x] 抽取事件聚合公共读取逻辑，统一过滤语义
- [x] 补充增量水位与 outcome 汇总单元测试
- [x] 完成验证并同步总项目文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive_federated.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_federated_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.37 能力实现、验证与文档同步完成）
- 下一步：P3.38 已完成，推进 P3.39（待规划）
- 最近更新：2026-02-11
