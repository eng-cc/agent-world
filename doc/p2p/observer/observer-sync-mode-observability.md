# Agent World Runtime：Observer 同步源策略可观测性（设计文档）

## 目标
- 为 `ObserverClient` 的模式化同步接口补充“是否触发回退”的显式观测结果，降低故障排查成本。
- 保持现有接口行为不变，在不破坏兼容性的前提下新增可观测报告接口。
- 同时覆盖非 DHT 与 DHT 组合模式。

## 范围

### In Scope
- 为 `HeadSyncSourceMode` 增加可观测报告接口，输出：
  - `mode`
  - `drained/applied`（沿用 `HeadSyncReport`）
  - `fallback_used`
- 为 `HeadSyncSourceModeWithDht` 增加同类可观测报告接口。
- 在内部模式分发流程中记录是否发生回退。
- 增加单元测试覆盖“无回退/触发回退”路径。

### Out of Scope
- 引入全局 metrics 系统（Prometheus/OpenTelemetry）。
- 持久化回退统计数据。
- 对现有 `HeadSyncReport` 结构做破坏性字段变更。

## 接口 / 数据

### 新增报告结构（草案）
- `HeadSyncModeReport`：
  - `mode: HeadSyncSourceMode`
  - `report: HeadSyncReport`
  - `fallback_used: bool`
- `HeadSyncModeWithDhtReport`：
  - `mode: HeadSyncSourceModeWithDht`
  - `report: HeadSyncReport`
  - `fallback_used: bool`

### 新增接口（草案）
- `sync_heads_with_mode_observed_report`
- `sync_heads_with_dht_mode_observed_report`

## 里程碑
- OSMO-1：设计文档与项目管理文档落地。
- OSMO-2：实现可观测报告结构与接口。
- OSMO-3：补齐测试并完成 `agent_world_net` 回归。
- OSMO-4：回写状态文档与 devlog。

## 风险
- 若回退标识计算位置不一致，可能与实际执行路径偏离，需要统一在模式分发层计算。
- 新增报告接口若与旧接口语义不清，可能导致调用方误用，需要保持命名直观。
