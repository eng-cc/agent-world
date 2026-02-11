# Agent World Runtime：成员目录吊销死信回放策略治理恢复演练告警事件总线增量水位自动推进与空拉取幂等（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.md`）
- [x] 输出项目管理文档（本文件）
- [x] 实现增量拉取 + 下一水位联动接口
- [x] 实现水位单调不回退语义（空拉取保持原水位）
- [x] 补充分批拉取/空拉取幂等测试
- [x] 完成验证并同步总项目文档/devlog

## 依赖
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.md`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery/replay_archive_federated.rs`
- `crates/agent_world/src/runtime/distributed_membership_sync/recovery_replay_federated_tests.rs`

## 状态
- 当前阶段：MR4 完成（P3.38 能力实现、验证与文档同步完成）
- 下一步：推进 P3.39（待规划）
- 最近更新：2026-02-11
