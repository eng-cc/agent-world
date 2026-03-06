# Agent World Simulator：Viewer Chat 独立最右侧 Panel（设计文档）

审计轮次: 4
- 对应项目管理文档: doc/world-simulator/viewer/viewer-chat-dedicated-right-panel.prd.project.md

## 1. Executive Summary
- 将聊天相关能力从当前综合右侧面板中拆分，迁移为独立 Chat Panel。
- Chat Panel 固定在界面最右侧，避免与观察/控制模块混排。
- 保持现有 `AgentChat` 协议与会话聚合逻辑不变，改造仅限 Viewer UI 与输入命中边界。

## 2. User Experience & Functionality

### 范围内
- `agent_world_viewer` EGUI 布局改造：
  - 新增独立 `Chat SidePanel`（最右侧）。
  - 现有综合右侧面板移除聊天内容渲染。
- 面板宽度状态改造：
  - 将“右侧占用宽度”更新为“综合面板宽度 + Chat Panel 宽度”，用于 3D 输入边界。
- `show_chat` 可见性语义保留：
  - 关闭时不渲染 Chat Panel，避免占用右侧空间。
- 补充/更新测试：
  - 3D 输入命中边界在双面板下仍正确。
  - Chat Panel 开关对右侧占用宽度生效。

### 范围外
- 不改动 `ViewerRequest::AgentChat` 与 `ViewerResponse::AgentChat*` 协议定义。
- 不改动 chat 会话切分算法与消息气泡样式。
- 不改动 Prompt Ops 的业务流程。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 复用现有 `RightPanelModuleVisibilityState.show_chat` 控制 Chat Panel 显隐。
- 复用现有 `AgentChatDraftState` 与 `ChatInputFocusSignal`。
- `RightPanelWidthState.width_px` 语义调整为“右侧总占用宽度”，由两个 SidePanel 宽度合并写回。

## 5. Risks & Roadmap
- M1：文档与任务拆解完成。
- M2：EGUI 双右侧面板布局改造完成（Chat 独立最右侧）。
- M3：输入命中边界与测试回归完成。
- M4：手册/devlog/项目文档收口完成。

### Technical Risks
- 风险：双面板下右侧总宽度变化可能影响 3D 视口可操作区域。
  - 缓解：统一使用“总占用宽度”进行相机与拾取边界判定。
- 风险：Prompt Ops 模式下 Chat 显示策略不一致可能引发认知差异。
  - 缓解：保持当前行为（Prompt Ops 下不展示 Chat），避免引入新交互分支。
- 风险：窄屏下双面板挤压可视区。
  - 缓解：限制默认/最小宽度并允许用户拖拽调整。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
