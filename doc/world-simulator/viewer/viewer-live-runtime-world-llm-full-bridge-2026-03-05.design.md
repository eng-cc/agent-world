# Viewer Live Runtime World LLM 全桥接设计（2026-03-05）

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.project.md`

## 1. 设计定位
定义 Viewer live 与 runtime world 之间的 LLM 全桥接链路、状态同步与错误恢复边界。

## 2. 设计结构
- 桥接层：viewer/live/runtime world 三方状态与事件桥接。
- LLM 层：Prompt/上下文/结果在桥接链路中的传递与回执。
- 恢复层：桥接断线、重连与重放的一致性处理。

## 3. 关键接口 / 入口
- viewer live / runtime world 之间的桥接消息
- LLM 触发、结果回传与状态观测接口
- 错误签名与重连控制入口

## 4. 约束与边界
- 不能让桥接层重写业务语义，只负责传递与编排。
- LLM 结果必须与 world 状态更新保持可追溯关联。
- 断线重连后不能产生重复执行。

## 5. 设计演进计划
- 先完成 Design 补齐与互链回写。
- 再沿项目文档推进实现与验证。
