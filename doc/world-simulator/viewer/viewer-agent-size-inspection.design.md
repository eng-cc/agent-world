# Viewer Agent 尺寸检查设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-agent-size-inspection.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-agent-size-inspection.project.md`

## 1. 设计定位
定义 Viewer 对 Agent 尺寸映射的检查与可视化诊断方案，使 `height_cm` 等世界语义在渲染层的表现可被快速核验、对比与回归。

## 2. 设计结构
- 尺寸映射层：明确世界尺寸到渲染尺寸的换算与 clamp 规则。
- 检查展示层：在 UI 或调试标记中暴露尺寸检查结果与关键数值。
- 回归对比层：通过测试与截图验证不同身高 Agent 的相对比例正确。
- 诊断层：为极小、极大、异常输入保留明确边界行为。

## 3. 关键接口 / 入口
- `height_cm`
- Agent 渲染 `scale`
- 尺寸检查 UI / 调试信息
- 回归截图与比例断言

## 4. 约束与边界
- 检查能力只用于验证渲染正确性，不改变协议和玩法语义。
- clamp 规则要稳定，避免不同场景下出现漂移。
- 极端输入必须可诊断，不静默掩盖。
- 本阶段不扩展到设施/Location 的全量尺寸审计。

## 5. 设计演进计划
- 先冻结尺寸映射与边界规则。
- 再补 UI/调试检查入口与对比回归。
- 最后将尺寸检查纳入截图与测试闭环。
