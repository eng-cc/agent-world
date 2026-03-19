# oasis7 Simulator：LLM Prompt ModuleCall Effect/Receipt 可回放链路（设计文档）设计

- 对应需求文档: `doc/world-simulator/llm/llm-prompt-effect-receipt.prd.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-prompt-effect-receipt.project.md`

## 1. 设计定位
定义 LLM Prompt ModuleCall Effect/Receipt 可回放链路设计，让 prompt 决策、工具效果与收据形成闭环证据。

## 2. 设计结构
- prompt 触发层：记录 prompt 生成的模块调用意图。
- effect/receipt 层：沉淀执行效果与收据结构。
- 回放链路层：支持按事件重放 prompt 到 receipt 的全过程。
- 审计对账层：比对预期调用与实际效果差异。

## 3. 关键接口 / 入口
- prompt 调用记录
- effect/receipt 结构
- 回放入口
- 差异对账记录

## 4. 约束与边界
- 收据必须与实际执行结果一致。
- 回放链路不得丢失关键上下文。
- 不在本专题重构完整审计系统。

## 5. 设计演进计划
- 先统一 effect/receipt 结构。
- 再打通回放链路。
- 最后补差异对账与回归。
