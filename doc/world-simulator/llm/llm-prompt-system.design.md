# oasis7 Simulator：Agent-LLM Prompt 模块交互系统（设计文档）设计

- 对应需求文档: `doc/world-simulator/llm/llm-prompt-system.prd.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-prompt-system.project.md`

## 1. 设计定位
定义 Agent-LLM Prompt 模块交互系统设计，统一 prompt 结构、模块语义和上下文装配逻辑。

## 2. 设计结构
- 系统 prompt 层：定义角色、规则与世界上下文主干。
- 模块交互层：让 prompt 与模块能力、动作语义对齐。
- 上下文装配层：收敛记忆、观察与任务状态输入。
- 治理回归层：保证 prompt 变更可审计、可比较。

## 3. 关键接口 / 入口
- 系统 prompt 模板
- 模块能力描述
- 上下文装配入口
- prompt 回归基线

## 4. 约束与边界
- prompt 变更必须可追踪。
- 模块语义描述需与实际能力一致。
- 不在本专题扩展前端对话 UI。

## 5. 设计演进计划
- 先收敛系统 prompt。
- 再对齐模块交互语义。
- 最后建立回归基线。
