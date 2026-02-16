# Agent World Runtime：Observer 同步源统计桥接（设计文档）

## 目标
- 将 `ObserverRuntimeMetrics` 从“可手动记录”推进到“跟随同步流程自动记录”，降低上层 runtime 接入复杂度。
- 为非 DHT 与 DHT 组合模式提供一致的“同步 + 计数”入口。
- 保持现有同步 API 兼容，新增桥接接口而非破坏性改动。

## 范围

### In Scope
- 在 `ObserverClient` 增加“执行同步并自动写入 metrics”的桥接接口。
- 支持单轮同步桥接与多轮 follow 桥接。
- follow 桥接沿用既有 `max_rounds` 终止语义，内部按轮记录 metrics。
- 补齐单元测试覆盖：
  - 回退路径下 fallback 计数自动增长。
  - follow 场景下多轮计数与 `HeadFollowReport` 行为一致。

### Out of Scope
- viewer 面板 UI 直接渲染与展示。
- 指标持久化、导出（Prometheus/OTel）。
- 全局调度器级别的采样频率管理。

## 接口 / 数据

### 新增桥接接口（草案）
- `sync_heads_with_mode_observed_report_and_record`
- `sync_heads_with_dht_mode_observed_report_and_record`
- `follow_heads_with_mode_and_metrics`
- `follow_heads_with_dht_mode_and_metrics`

### 语义约束
- 每次桥接接口成功产出一轮报告，必定调用对应 `record_*`。
- `follow_*` 桥接内部按轮记录，最终返回原有 `HeadFollowReport`，不改变报告聚合规则。
- 统计结构仍由调用方持有（`&mut ObserverRuntimeMetrics`），避免隐式全局状态。

## 里程碑
- OSMB-1：设计文档与项目管理文档落地。
- OSMB-2：实现桥接接口与导出。
- OSMB-3：补齐桥接接口测试并完成 `agent_world_net` 回归。
- OSMB-4：回写状态文档与 devlog 收口。

## 风险
- 桥接接口命名过长或语义不清，可能造成调用方误选；需保持“observed + record/metrics”可辨识。
- follow 场景若记录时机与原有轮次判定不一致，可能导致统计偏差；需复用现有 `follow_head_sync` 语义。
