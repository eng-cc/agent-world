# Agent World Simulator：LLM Prompt 组装重构与多步决策机制（设计文档）设计

- 对应需求文档: `doc/world-simulator/llm/llm-prompt-multi-step-orchestration.prd.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-prompt-multi-step-orchestration.project.md`

## 1. 设计定位
定义 LLM Prompt 组装重构与多步决策机制设计，支持多阶段推理、工具调用与结果拼装。

## 2. 设计结构
- prompt 组装层：重构系统、上下文与任务 prompt 的拼装方式。
- 多步决策层：把单轮生成扩展为多步计划与执行。
- 工具编排层：在步骤间插入工具调用、观察与再规划。
- 结果收敛层：把多步过程汇总为稳定输出与状态更新。

## 3. 关键接口 / 入口
- prompt 组装入口
- 多步计划状态
- 工具编排接口
- 结果收敛输出

## 4. 约束与边界
- 多步编排必须有明确终止条件。
- 步骤间状态传递需可追踪。
- 不在本专题扩展通用自治代理框架。

## 5. 设计演进计划
- 先重构 prompt 组装。
- 再引入多步编排。
- 最后固定结果收敛与回归。
