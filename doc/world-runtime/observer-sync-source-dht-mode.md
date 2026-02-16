# Agent World Runtime：Observer 同步源策略化（DHT 组合链路，设计文档）

## 目标
- 为 `ObserverClient` 增加 DHT 组合链路下的 head 同步源策略，显式控制“网络+DHT路径”与“路径索引路径”的切换。
- 在网络+DHT链路失败时支持回退到路径索引，提升本地恢复鲁棒性。
- 保持现有 API 兼容，不影响已落地的非 DHT 策略接口。

## 范围

### In Scope
- 定义 `HeadSyncSourceModeWithDht` 策略枚举。
- 在 `ObserverClient` 增加 DHT 组合模式接口：同步/报告/结果/跟随循环。
- 支持模式：
  - `NetworkWithDhtOnly`
  - `PathIndexOnly`
  - `NetworkWithDhtThenPathIndex`
- 补齐单元测试，覆盖 DHT 模式与失败回退路径索引。

### Out of Scope
- 新增全局配置中心或动态热更新策略。
- 指标上报系统（Prometheus/OpenTelemetry）接入。
- 多级回退链（如 Network -> DHT -> Network(no DHT) -> PathIndex）。

## 接口 / 数据

### 策略枚举（草案）
- `HeadSyncSourceModeWithDht::NetworkWithDhtOnly`
- `HeadSyncSourceModeWithDht::PathIndexOnly`
- `HeadSyncSourceModeWithDht::NetworkWithDhtThenPathIndex`

### 语义约束
- `NetworkWithDhtOnly`：仅走现有 `sync_from_heads_with_dht` 链路，失败直接返回。
- `PathIndexOnly`：仅走路径索引读取。
- `NetworkWithDhtThenPathIndex`：先走网络+DHT；仅在该链路报错时回退路径索引。

## 里程碑
- OSDM-1：设计文档与项目管理文档落地。
- OSDM-2：实现 `HeadSyncSourceModeWithDht` 与 `ObserverClient` 模式化接口。
- OSDM-3：补齐单元测试并完成 `agent_world_net` 回归。
- OSDM-4：回写状态文档与 devlog。

## 风险
- DHT 失败回退路径索引可能掩盖上游网络问题，需要保留错误上下文。
- 模式枚举扩展后若命名不清晰，调用方容易误用，需要保持语义直观。
