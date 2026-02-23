# Agent World Runtime：分布式存储自愈轮询 Runtime 接线（2026-02-23）

## 目标
- 将已实现的 `run_replica_maintenance_poll` 接入 `NodeRuntime` 周期循环，实现自动触发副本维护。
- 保持“无单机完整数据假设”：节点仅在目标副本为本节点时执行拉取与落盘，不引入单机全量回退路径。
- 保持现有共识/复制主链路稳定，轮询失败不影响主 tick 的推进，仅记录错误。

## 范围

### In Scope
- `agent_world_node` 新增 Runtime 级副本维护配置模型（启用开关、采样窗口、维护策略、轮询策略）。
- 在 `NodeRuntime` worker tick 中接线维护轮询：
  - 按 `poll_interval_ms` 判断是否执行。
  - 采集候选 content hash（来自本地复制热数据窗口）。
  - 调用 `run_replica_maintenance_poll` 执行计划+任务。
- 新增 Node 侧执行器：
  - 基于复制网络拉取源 provider blob。
  - 写入本地 CAS。
  - 仅允许执行 `target_provider_id == local_node_id` 的任务。
- 补齐单测：启用后可触发执行；未配置 DHT 时跳过且不影响 tick；非法策略被拒绝。

### Out of Scope
- 远端目标节点任务委派与跨节点协同执行协议（本轮仅做本地目标可执行闭环）。
- 全局调度仲裁与冲突解决（并发多节点 planning 冲突）。
- 维护结果持久化审计索引（本轮仅依赖 runtime 内错误观测）。

## 接口/数据

### 1) Runtime 配置
- `NodeReplicaMaintenanceConfig`
  - `enabled: bool`
  - `max_content_hash_samples_per_round: usize`
  - `target_replicas_per_blob: usize`
  - `max_repairs_per_round: usize`
  - `max_rebalances_per_round: usize`
  - `rebalance_source_load_min_per_mille: u16`
  - `rebalance_target_load_max_per_mille: u16`
  - `poll_interval_ms: i64`

### 2) Runtime 状态扩展
- `RuntimeState` 增加副本维护轮询状态与最近一轮摘要（最小必要字段）。

### 3) DHT 句柄接线
- `NodeRuntime` 新增可选 DHT 句柄注入接口；未注入时维护轮询自动降级为跳过。

### 4) 轮询执行语义
- 轮询仅在满足以下条件时运行：
  - 配置启用；
  - replication runtime 可用；
  - replication network 可用；
  - DHT 句柄可用；
  - 本轮可采样到 content hash。
- 轮询失败写入 `last_error`，不阻断 tick 主流程。

## 里程碑
- M0：设计与任务拆解。
- M1：Runtime 配置/状态/轮询接线与执行器实现。
- M2：单测与跨 crate 回归，文档日志收口。

## 风险
- 风险：多节点并发规划产生重复任务。
  - 缓解：执行器只接收本地 target 任务，天然抑制跨节点误执行；重复写同 hash 在 CAS 层幂等。
- 风险：provider 定向请求不可用导致拉取不稳定。
  - 缓解：先尝试按 provider 定向请求，失败时按网络默认请求兜底并保留错误。
- 风险：轮询耗时影响 tick 周期。
  - 缓解：通过 `max_content_hash_samples_per_round` 与维护策略配额限制单轮开销。

## 当前状态
- 状态：已完成
- 已完成：M0、M1、M2
- 进行中：无
- 未开始：无
