# Agent World Runtime：分布式存储自愈定时轮询（2026-02-23）

## 目标
- 在现有 `plan_replica_maintenance + execute_replica_maintenance_plan` 基础上补齐按周期触发的轮询入口。
- 让自愈控制面从“可手工触发”升级为“可自动周期巡检触发”，避免长期运行依赖人工干预。
- 保持无单机完整依赖假设：轮询仅驱动维护计划，不引入对单机全量数据的回退路径。

## 范围

### In Scope
- `agent_world_net::replica_maintenance` 新增轮询模型：
  - 轮询策略（间隔毫秒）。
  - 轮询状态（上次执行时间）。
  - 轮询结果（计划+执行报告）。
- 新增轮询入口：
  - 根据 `now_ms` 与 `last_polled_at_ms` 判断是否到期；到期则执行计划与维护任务。
- 增补 `agent_world_net` 单测：
  - 首轮执行。
  - 间隔未到时跳过。
  - 非法轮询策略拦截。

### Out of Scope
- NodeRuntime 侧具体线程调度接线（本轮先提供可直接复用的轮询函数）。
- 真实跨机传输协议优化（沿用现有执行器抽象）。
- 多租户/多 world 的任务优先级编排。

## 接口/数据

### 1) 轮询策略
- `ReplicaMaintenancePollingPolicy`
  - `poll_interval_ms: i64`（默认 60_000）

### 2) 轮询状态
- `ReplicaMaintenancePollingState`
  - `last_polled_at_ms: Option<i64>`

### 3) 轮询入口
- `run_replica_maintenance_poll(...) -> Result<Option<ReplicaMaintenanceRoundResult>, WorldError>`
  - 未到间隔：返回 `Ok(None)`。
  - 到间隔：执行 `plan + execute`，返回 `Ok(Some(...))`。

### 4) 轮询结果
- `ReplicaMaintenanceRoundResult`
  - `polled_at_ms`
  - `plan`
  - `report`

## 里程碑
- M0：设计与任务拆解。
- M1：轮询策略/状态/入口与单测落地。
- M2：回归、文档与日志收口。

## 风险
- 风险：轮询周期过短可能造成无效高频检查。
  - 缓解：策略要求 `poll_interval_ms > 0`，并在接口层提前拦截非法配置。
- 风险：时间回拨导致执行节奏异常。
  - 缓解：使用基于 `last_polled_at_ms` 的显式间隔判断，未到期则跳过。

## 当前状态
- 状态：进行中
- 已完成：M0
- 进行中：M1
- 未开始：M2
