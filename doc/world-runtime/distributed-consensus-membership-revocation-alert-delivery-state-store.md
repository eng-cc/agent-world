# Agent World Runtime：成员目录吊销告警上报与调度状态持久化（设计文档）

## 目标
- 将吊销对账异常告警从“内存结果”升级为“可上报 sink”，支持后续接入外部告警系统。
- 为对账调度状态提供统一 store 抽象，实现跨重启恢复调度进度。
- 保持与现有对账调度/告警评估接口兼容，最小改造上层调用。

## 范围

### In Scope（本次实现）
- 新增告警上报抽象与实现：
  - `MembershipRevocationAlertSink`
  - `InMemoryMembershipRevocationAlertSink`
  - `FileMembershipRevocationAlertSink`
- 新增调度状态存储抽象与实现：
  - `MembershipRevocationScheduleStateStore`
  - `InMemoryMembershipRevocationScheduleStateStore`
  - `FileMembershipRevocationScheduleStateStore`
- 新增接口：
  - `emit_revocation_reconcile_alerts(...)`
  - `run_revocation_reconcile_schedule_with_store_and_alerts(...)`
- 文件实现使用 JSON/JSONL 持久化（按 world/node 分片）。

### Out of Scope（本次不做）
- 对接外部告警服务（Webhook/IM/邮件）。
- 多副本状态 store 的分布式一致性（锁/选主）。
- 告警去重抑制、降噪策略。

## 接口 / 数据

### 告警上报
- `MembershipRevocationAlertSink::emit(alert)`
- `InMemoryMembershipRevocationAlertSink::list()`
- `FileMembershipRevocationAlertSink::list(world_id)`

### 调度状态持久化
- `MembershipRevocationScheduleStateStore::load(world_id, node_id)`
- `MembershipRevocationScheduleStateStore::save(world_id, node_id, state)`
- `MembershipRevocationReconcileScheduleState`
  - `last_checkpoint_at_ms`
  - `last_reconcile_at_ms`

### 编排入口
- `run_revocation_reconcile_schedule_with_store_and_alerts(...)`
  - 从 store 读取状态
  - 执行 schedule（checkpoint/reconcile）
  - 写回最新状态
  - 评估并上报告警

## 里程碑
- **MR1**：设计/项目文档完成。
- **MR2**：告警 sink 抽象与文件/内存实现完成。
- **MR3**：调度状态 store 抽象与文件/内存实现完成。
- **MR4**：编排入口、单测、导出与总文档更新完成。

## 风险
- 本地文件存储在高并发写入场景可能存在竞争，需要上层串行化调度。
- 仅按 world/node 维度持久化，若节点标识漂移会造成状态分裂。
- 告警无去重策略时，持续异常会导致 JSONL 快速增长。
