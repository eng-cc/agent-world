# Agent World Runtime：Observer 同步源运行态统计（设计文档）

## 目标
- 将 `ObserverClient` 已有的模式化可观测报告接入运行态统计，形成“持续监控”基础闭环。
- 统一沉淀非 DHT 与 DHT 组合模式下的核心计数：`total`、`applied`、`fallback`。
- 保持现有同步 API 不变，以新增结构与接口方式提供统计能力。

## 范围

### In Scope
- 在 `agent_world_net` 新增 observer 运行态统计模块（内存计数）。
- 提供针对 `HeadSyncModeReport` 与 `HeadSyncModeWithDhtReport` 的记录接口。
- 提供快照读取接口，便于上层 runtime/面板周期拉取并展示。
- 补充单元测试，覆盖各模式计数正确性与回退计数。

### Out of Scope
- Prometheus/OpenTelemetry exporter。
- 跨进程或落盘持久化统计。
- 告警规则引擎与阈值策略。

## 接口 / 数据

### 计数维度
- `total`: 收到一次模式化同步报告即 +1。
- `applied`: 报告 `report.applied.is_some()` 时 +1。
- `fallback`: 报告 `fallback_used == true` 时 +1。

### 新增结构（草案）
- `ObserverModeCounters`
  - `total: u64`
  - `applied: u64`
  - `fallback: u64`
- `ObserverModeRuntimeMetricsSnapshot`
  - `network_only: ObserverModeCounters`
  - `path_index_only: ObserverModeCounters`
  - `network_then_path_index: ObserverModeCounters`
- `ObserverModeWithDhtRuntimeMetricsSnapshot`
  - `network_with_dht_only: ObserverModeCounters`
  - `path_index_only: ObserverModeCounters`
  - `network_with_dht_then_path_index: ObserverModeCounters`
- `ObserverRuntimeMetricsSnapshot`
  - `mode: ObserverModeRuntimeMetricsSnapshot`
  - `mode_with_dht: ObserverModeWithDhtRuntimeMetricsSnapshot`

### 新增接口（草案）
- `ObserverRuntimeMetrics::record_mode_report(&HeadSyncModeReport)`
- `ObserverRuntimeMetrics::record_mode_with_dht_report(&HeadSyncModeWithDhtReport)`
- `ObserverRuntimeMetrics::snapshot() -> ObserverRuntimeMetricsSnapshot`

## 里程碑
- OSRM-1：设计文档与项目管理文档落地。
- OSRM-2：实现运行态统计结构与导出接口。
- OSRM-3：补齐单元测试并完成 `agent_world_net` 回归。
- OSRM-4：回写状态文档与 devlog 收口。

## 风险
- 统计语义若与调用方预期不一致（例如 `total` 是否按轮次或按 head 条目），会导致面板误判；需在文档中固定“按报告次数计数”。
- 若后续扩展更多模式，存在字段膨胀风险；需要保持结构可扩展并保持向后兼容。
