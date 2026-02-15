# Agent World Simulator：Viewer 快速定位 Agent 按钮（设计文档）

## 目标
- 在 Viewer 右侧「事件联动」区域新增“快速定位 Agent”按钮，降低在对象密集场景中手动查找 Agent 的成本。
- 复用现有选中/高亮链路，确保定位行为可预测、可测试。
- 与现有事件联动能力兼容，不引入新的协议或服务依赖。

## 范围

### 范围内
- `agent_world_viewer` 新增 Agent 快速定位动作：优先定位当前已选中的 Agent，否则定位当前场景字典序第一个 Agent。
- `agent_world_viewer` 右侧 Egui 面板「事件联动」新增按钮并接入该动作。
- 兼容旧 UI 控件路径（Bevy UI 按钮体系）并补齐标签更新。
- 补充单元测试（动作行为、标签文案）。

### 范围外
- 不改 viewer/server 协议。
- 不新增路径规划或镜头动画。
- 不扩展到 asset/power/chunk 的“快速定位”按钮（后续按需补充）。

## 接口 / 数据

### UI 入口
- 区域：右侧 `Event Link / 事件联动` 模块
- 按钮文案：
  - 中文：`定位 Agent`
  - 英文：`Locate Agent`

### 动作语义
- 输入：
  - `ViewerSelection`
  - `Viewer3dScene.agent_entities`
  - `Viewer3dConfig`
- 行为：
  1. 若当前选中对象是 Agent 且仍在场景中，则直接定位该 Agent。
  2. 否则从 `agent_entities` 取字典序第一个 Agent 作为目标。
  3. 通过既有 `apply_selection` 完成选中与高亮。
  4. 更新联动状态文本（成功或失败原因）。
- 失败兜底：
  - 无 Agent 时输出可读提示，不触发 panic。

## 里程碑
- **QAG1**：设计文档与项目文档。
- **QAG2**：动作函数与 UI 按钮接入（Egui + 兼容旧 UI）。
- **QAG3**：测试回归与文档收口。

## 风险
- 场景重建后实体引用可能失效，需要在动作中做存在性校验。
- 极端窄面板下按钮换行可能影响可读性，需复用现有 `horizontal_wrapped` 布局。
- 若后续引入“多 Agent 分组过滤”，当前“字典序首个 Agent”策略可能需要升级。
