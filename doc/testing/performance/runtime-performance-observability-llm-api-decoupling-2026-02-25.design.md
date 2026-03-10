# Agent World Runtime：LLM API 延迟与代码执行耗时解耦（2026-02-25）设计

- 对应需求文档: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.prd.md`
- 对应项目管理文档: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.project.md`

## 1. 设计定位
定义性能测试与可观测性专题设计，统一采集、分析、瓶颈定位与方法论闭环。

## 2. 设计结构
- 采集基础层：定义性能指标、采样点与日志/metrics 输出。
- 分析方法层：沉淀瓶颈定位、解耦分析与对比方法。
- 专题接线层：把 runtime、viewer 或 LLM 延迟观测接入同一方法论。
- 回归基线层：形成性能测试基线、报告与验收结论。

## 3. 关键接口 / 入口
- 性能采集入口
- metrics/log 分析点
- 专题对齐方法论
- 性能回归基线

## 4. 约束与边界
- 性能指标必须与用户体验或运行时行为相关。
- 分析结论需可复现、可比较。
- 不在本专题扩展完整 profiling 平台。

## 5. 设计演进计划
- 先定义性能指标与采样点。
- 再补分析方法与专题接线。
- 最后固化基线与回归。
