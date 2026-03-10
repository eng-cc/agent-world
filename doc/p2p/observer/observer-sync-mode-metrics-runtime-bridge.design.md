# Agent World Runtime：Observer 同步源统计桥接设计

- 对应需求文档: `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.prd.md`
- 对应项目管理文档: `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.project.md`

## 1. 设计定位
定义 Observer 同步源统计桥接设计，把运行时统计结果稳定桥接到 observer 指标与对外读数。

## 2. 设计结构
- 运行时采集层：采集 observer 同步源相关运行时状态。
- 指标桥接层：把运行时数据映射为稳定 metrics 输出。
- 读数对齐层：统一 dashboard、日志与指标含义。
- 回归校验层：验证桥接后读数与实际同步行为一致。

## 3. 关键接口 / 入口
- observer runtime metrics
- 指标桥接入口
- dashboard/log 字段
- 桥接回归用例

## 4. 约束与边界
- 指标语义必须与运行时状态一一对应。
- 桥接过程不得引入重复或冲突读数。
- 不在本专题扩展新的监控平台。

## 5. 设计演进计划
- 先固化运行态统计字段。
- 再补 metrics bridge。
- 最后回写可观测文档。
