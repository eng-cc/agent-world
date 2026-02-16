# Agent World Runtime：Observer 同步源策略化（设计文档）

## 目标
- 为 `ObserverClient` 增加可配置的 head 同步源策略，显式控制“网络路径”与“路径索引路径”的使用方式。
- 支持网络失败时回退路径索引，提升本地恢复弹性。
- 保持现有 API 可用，不破坏当前调用方。

## 范围

### In Scope
- 定义 `HeadSyncSourceMode` 策略枚举（非 DHT 链路）。
- 在 `ObserverClient` 增加 `sync_heads_with_mode` 与对应报告/结果/循环接口。
- 支持模式：
  - `NetworkOnly`
  - `PathIndexOnly`
  - `NetworkThenPathIndex`
- 补齐策略模式单元测试（重点覆盖网络失败回退路径索引）。

### Out of Scope
- DHT 链路下的策略组合（如 `NetworkWithDhtThenPathIndex`）。
- 全局配置中心或动态热更新配置。
- 指标埋点/告警联动。

## 接口 / 数据

### 策略枚举（草案）
- `HeadSyncSourceMode::NetworkOnly`
- `HeadSyncSourceMode::PathIndexOnly`
- `HeadSyncSourceMode::NetworkThenPathIndex`

### 语义约束
- `NetworkOnly`：仅走现有网络恢复链路，失败直接返回错误。
- `PathIndexOnly`：仅走路径索引恢复链路。
- `NetworkThenPathIndex`：先走网络；仅在网络恢复报错时回退路径索引。

## 里程碑
- OSSM-1：设计文档与项目管理文档落地。
- OSSM-2：策略枚举与 `ObserverClient` 模式化接口实现。
- OSSM-3：补齐测试并完成 `agent_world_net` 回归。
- OSSM-4：状态文档与 devlog 收口。

## 风险
- 模式过多可能引入调用歧义，需保持命名清晰。
- 回退策略若吞掉网络错误，定位问题成本会提升，需要保留错误上下文。
