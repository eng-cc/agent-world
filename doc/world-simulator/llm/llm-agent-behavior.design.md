# Agent World Simulator：LLM 驱动 Agent 行为落地（设计文档）设计

- 对应需求文档: `doc/world-simulator/llm/llm-agent-behavior.prd.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-agent-behavior.project.md`

## 1. 设计定位
定义 LLM 驱动 Agent 行为落地设计，统一感知输入、决策输出、模块调用与安全约束。

## 2. 设计结构
- 感知输入层：收集 agent 观察、上下文与目标。
- 决策生成层：由 LLM 产生行为建议与模块调用意图。
- 执行桥接层：把 LLM 输出约束到可执行动作与模块接口。
- 安全治理层：对越界行为、成本与失败进行限制。

## 3. 关键接口 / 入口
- agent 观察上下文
- LLM 决策输出
- 模块/动作执行入口
- 行为安全守卫

## 4. 约束与边界
- LLM 输出必须经过结构化约束后执行。
- 失败与拒绝需要明确反馈。
- 不在本专题直接定义所有具体玩法策略。

## 5. 设计演进计划
- 先固定 agent 行为输入输出。
- 再接模块执行桥接。
- 最后补安全治理与回归。
