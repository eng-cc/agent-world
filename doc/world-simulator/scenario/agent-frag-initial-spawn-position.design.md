# Agent World Simulator：Agent Frag 初始站位优化（设计文档）设计

- 对应需求文档: `doc/world-simulator/scenario/agent-frag-initial-spawn-position.prd.md`
- 对应项目管理文档: `doc/world-simulator/scenario/agent-frag-initial-spawn-position.project.md`

## 1. 设计定位
定义 Agent Frag 初始站位优化设计，统一新生 Agent 在碎片场景中的出生位置选择、避让规则与可重复性。

## 2. 设计结构
- 出生采样层：根据场景与碎片布局生成候选站位。
- 避让约束层：避免与障碍、边界或已有实体发生冲突。
- 落位确认层：在运行时初始化阶段确定最终出生点。
- 回归验证层：校验不同 seed 与碎片布局下的站位稳定性。

## 3. 关键接口 / 入口
- 初始站位生成入口
- 碎片/边界约束读取
- Agent 初始化落位
- 站位回归用例

## 4. 约束与边界
- 同一 seed 下站位应可复现。
- 出生点选择不得破坏既有场景边界。
- 不在本专题扩展完整导航系统。

## 5. 设计演进计划
- 先固化站位选择规则。
- 再补避让与边界守卫。
- 最后沉淀场景回归。
