# Agent World Runtime：成员目录吊销协同状态外部存储与告警恢复机制（设计文档）

## 目标
- 为成员目录吊销调度协同锁增加外部状态存储抽象，支持跨进程/跨重启延续协调状态。
- 在告警发送链路中增加“失败缓冲 + 恢复重放”机制，降低下游不可用时的告警丢失风险。
- 与既有调度、去重、协同编排接口保持兼容并支持渐进接入。

## 范围

### In Scope（本次实现）
- 新增协同状态存储抽象与实现：
  - `MembershipRevocationCoordinatorStateStore`
  - `InMemoryMembershipRevocationCoordinatorStateStore`
  - `FileMembershipRevocationCoordinatorStateStore`
  - `MembershipRevocationCoordinatorLeaseState`
- 新增基于 store 的协同器实现：
  - `StoreBackedMembershipRevocationScheduleCoordinator`
- 新增告警恢复存储抽象与实现：
  - `MembershipRevocationAlertRecoveryStore`
  - `InMemoryMembershipRevocationAlertRecoveryStore`
  - `FileMembershipRevocationAlertRecoveryStore`
- 新增恢复上报与协同编排入口：
  - `emit_revocation_reconcile_alerts_with_recovery(...)`
  - `run_revocation_reconcile_coordinated_with_recovery(...)`
- 新增运行报告：
  - `MembershipRevocationAlertRecoveryReport`
  - `MembershipRevocationCoordinatedRecoveryRunReport`

### Out of Scope（本次不做）
- 外部一致性存储（Redis/etcd）正式适配与高可用部署。
- 告警 ACK/重试退避策略分级控制。
- 协同状态和恢复队列加密存储与访问审计。

## 接口 / 数据

### 协同状态外部存储
- `MembershipRevocationCoordinatorStateStore`
  - `load(world_id)`
  - `save(world_id, lease_state)`
  - `clear(world_id)`
- `MembershipRevocationCoordinatorLeaseState`
  - `holder_node_id`
  - `expires_at_ms`

### 告警恢复机制
- `MembershipRevocationAlertRecoveryStore`
  - `load_pending(world_id, node_id)`
  - `save_pending(world_id, node_id, alerts)`
- `emit_revocation_reconcile_alerts_with_recovery(...)`
  - 先重放 pending，再发送新告警
  - 发送失败剩余告警回写 pending

### 协同恢复编排
- `run_revocation_reconcile_coordinated_with_recovery(...)`
  - 协调器抢锁
  - 读取并执行 schedule 状态
  - 评估/去重告警
  - 按恢复机制发送并缓存失败告警
  - 返回 `MembershipRevocationCoordinatedRecoveryRunReport`

## 里程碑
- **MR1**：设计/项目文档完成。
- **MR2**：协同状态外部存储抽象与实现完成。
- **MR3**：告警恢复存储与恢复发送入口完成。
- **MR4**：协同恢复编排、测试、导出与总文档更新完成。

## 风险
- 文件存储在并发写入场景仍有竞争风险，生产需配合外部锁或串行调度。
- 恢复队列未设置容量上限时，长期下游故障可能导致积压增长。
- 节点时钟偏移会影响协同租约过期判断的准确性。
