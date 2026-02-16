> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销异常告警与对账调度自动化（设计文档）

## 目标
- 为成员目录吊销对账结果提供标准化异常告警结构，降低人工巡检成本。
- 提供可复用的对账调度状态机，支持按时间间隔自动执行 checkpoint 发布与对账收敛。
- 保持与现有 `membership.reconcile` 通道和 `MembershipRevocationReconcilePolicy` 兼容。

## 范围

### In Scope（本次实现）
- 新增异常告警策略与告警数据结构：
  - `MembershipRevocationAlertPolicy`
  - `MembershipRevocationAlertSeverity`
  - `MembershipRevocationAnomalyAlert`
- 新增告警评估接口：
  - `evaluate_revocation_reconcile_alerts(...)`
- 新增对账调度策略/状态/运行报告：
  - `MembershipRevocationReconcileSchedulePolicy`
  - `MembershipRevocationReconcileScheduleState`
  - `MembershipRevocationScheduledRunReport`
- 新增调度执行入口：
  - `run_revocation_reconcile_schedule(...)`
- 对调度策略做基础校验（interval 必须 > 0）并补充单元测试。

### Out of Scope（本次不做）
- 告警消息推送到外部系统（邮件、Webhook、IM）。
- 调度状态持久化到数据库或分布式 KV。
- 对账调度任务的跨进程 leader 选举。

## 接口 / 数据

### 异常告警
- `MembershipRevocationAlertPolicy`
  - `warn_diverged_threshold`
  - `critical_rejected_threshold`
- `MembershipRevocationAnomalyAlert`
  - `world_id`
  - `node_id`
  - `detected_at_ms`
  - `severity`
  - `code/message`
  - `drained/diverged/rejected`

### 调度自动化
- `MembershipRevocationReconcileSchedulePolicy`
  - `checkpoint_interval_ms`
  - `reconcile_interval_ms`
- `MembershipRevocationReconcileScheduleState`
  - `last_checkpoint_at_ms`
  - `last_reconcile_at_ms`
- `run_revocation_reconcile_schedule(...)`
  - 基于 interval 判定“是否到期”
  - 到期则执行 checkpoint 发布或对账收敛
  - 返回单轮执行报告，便于上层编排和可观测

## 里程碑
- **MR1**：设计/项目文档完成。
- **MR2**：异常告警策略与评估接口实现。
- **MR3**：对账调度策略/状态/执行入口实现。
- **MR4**：单测、导出接口、总文档与日志更新。

## 风险
- 阈值配置过低会造成告警噪音，过高会延迟异常发现。
- 仅基于本地时钟调度，跨节点时钟漂移会影响执行节奏。
- 未持久化调度状态时，进程重启后会触发一次“首次执行”行为。
