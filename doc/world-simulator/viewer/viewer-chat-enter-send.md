# Agent World Viewer：Chat 输入回车发送（设计文档）

## 目标
- 在 Viewer 右侧独立 Chat 面板中支持“回车发送”。
- 保持现有“点击发送按钮”行为不变。
- 保留多行输入能力：`Shift+Enter` 继续换行，不触发发送。

## 范围

### 范围内
- `agent_world_viewer` Chat 输入区交互改造：
  - 输入框聚焦时，`Enter` 触发发送（与按钮发送一致）。
  - `Shift+Enter` 不发送，用于换行。
- 补充单元测试，覆盖 Enter 发送判定逻辑。
- 更新 Viewer 使用手册（中英文）中的 Chat 输入说明。

### 范围外
- 不改动 `ViewerRequest::AgentChat` / `ViewerResponse::AgentChat*` 协议。
- 不改动 chat 线程聚合与消息气泡渲染逻辑。
- 不改动 Prompt Ops 面板的输入交互。

## 接口 / 数据
- 复用 `AgentChatDraftState` 的输入草稿与状态信息，不新增持久化结构。
- 发送链路继续走现有 `ViewerRequest::AgentChat`，仅新增 Enter 触发入口。

## 里程碑
- M1：文档与任务拆解完成。
- M2：Chat 输入区 Enter 发送实现完成。
- M3：测试与手册更新完成并收口。

## 风险
- 风险：`Enter` 与 IME 组合输入确认键冲突，可能误触发送。
  - 缓解：仅在“无修饰键”的 Enter 且输入框聚焦时触发，保留已有 IME 输入桥接与文本编辑状态判定。
- 风险：多行输入能力被削弱。
  - 缓解：明确 `Shift+Enter` 换行语义，并在手册中说明。
