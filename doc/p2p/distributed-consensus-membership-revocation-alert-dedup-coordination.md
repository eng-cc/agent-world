# Agent World Runtime：成员目录吊销告警抑制去重与调度多节点协同（设计文档）

## 目标
- 在吊销对账告警上报链路中增加去重抑制，降低持续异常场景的告警噪音。
- 为对账调度提供多节点协同入口，避免多个节点重复执行同一周期任务。
- 与现有 store/sink 调度编排接口兼容，支持增量接入。

## 范围

### In Scope（本次实现）
- 告警去重抑制策略与状态：
  - `MembershipRevocationAlertDedupPolicy`
  - `MembershipRevocationAlertDedupState`
  - `deduplicate_revocation_alerts(...)`
- 多节点调度协同抽象与内存实现：
  - `MembershipRevocationScheduleCoordinator`
  - `InMemoryMembershipRevocationScheduleCoordinator`
- 协同运行报告与编排入口：
  - `MembershipRevocationCoordinatedRunReport`
  - `run_revocation_reconcile_coordinated(...)`
- 单元测试覆盖去重抑制与协调锁行为。

### Out of Scope（本次不做）
- 协调锁持久化到外部 KV（Redis/etcd）。
- 分布式时钟漂移校正与 NTP 容错。
- 告警级联降噪（聚合、升级、恢复策略）。

## 接口 / 数据

### 告警去重
- `MembershipRevocationAlertDedupPolicy`
  - `suppress_window_ms`
- `MembershipRevocationAlertDedupState`
  - `last_emitted_at_by_key`
- `deduplicate_revocation_alerts(...)`
  - 输入候选告警列表
  - 基于 key + 时间窗口输出可发送告警

### 多节点协同
- `MembershipRevocationScheduleCoordinator`
  - `acquire(...)` / `release(...)`
- `MembershipRevocationCoordinatedRunReport`
  - `acquired`
  - `emitted_alerts`
  - `run_report`
- `run_revocation_reconcile_coordinated(...)`
  - 先协调获取执行权
  - 成功后执行 schedule/store/alert 编排
  - 可选应用告警去重

## 里程碑
- **MR1**：设计/项目文档完成。
- **MR2**：告警去重策略与状态实现。
- **MR3**：多节点协同抽象/内存实现与编排入口实现。
- **MR4**：测试、导出、总文档与日志更新。

## 风险
- 去重窗口配置过大会掩盖真实高频故障，过小则噪声抑制不足。
- 协同锁基于本地内存，仅适用于单进程模拟或测试环境。
- 节点异常退出未显式释放时，需要依赖 TTL 或下一轮协调回收。
