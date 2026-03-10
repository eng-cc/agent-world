# 间接控制链路 + WASM Tick 生命周期 + 长期记忆持久化（设计文档）设计

- 对应需求文档: `doc/world-simulator/llm/indirect-control-tick-lifecycle-long-term-memory.prd.md`
- 对应项目管理文档: `doc/world-simulator/llm/indirect-control-tick-lifecycle-long-term-memory.project.md`

## 1. 设计定位
定义间接控制链路、WASM Tick 生命周期与长期记忆持久化的一体化设计，让 LLM/模块行为能跨 tick 保持稳定上下文与可追溯状态。

## 2. 设计结构
- 间接控制层：把高层意图转换为可执行的 tick 级控制指令。
- 生命周期层：统一 WASM tick 前后状态、事件与执行边界。
- 长期记忆层：为 agent 保留可持久化、可检索的长期记忆载体。
- 回放审计层：保证控制、tick 与记忆变化可重放、可核查。

## 3. 关键接口 / 入口
- 间接控制输入/输出
- WASM tick 生命周期钩子
- 长期记忆存取入口
- 回放与审计记录

## 4. 约束与边界
- 间接控制不得绕开 tick 主循环语义。
- 长期记忆写入必须可追溯、可裁剪。
- 不在本专题扩展完整通用记忆平台。

## 5. 设计演进计划
- 先打通控制链路与 tick 生命周期。
- 再补长期记忆持久化。
- 最后固化回放与审计能力。
