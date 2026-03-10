# Viewer Chat 回车发送设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-chat-enter-send.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-chat-enter-send.project.md`

## 1. 设计定位
定义 Chat 输入区的键盘发送语义：`Enter` 发送、`Shift+Enter` 换行，在不破坏按钮发送与多行输入能力的前提下提升输入效率。

## 2. 设计结构
- 键盘判定层：在输入框聚焦且无修饰键时由 `Enter` 触发发送。
- 多行保留层：`Shift+Enter` 保持换行，不触发发送。
- 发送复用层：仍走现有 `ViewerRequest::AgentChat` 发送路径。
- 文档同步层：在手册中明确键盘语义，减少认知偏差。

## 3. 关键接口 / 入口
- Chat 输入框聚焦状态
- `Enter` / `Shift+Enter`
- `AgentChatDraftState`
- `ViewerRequest::AgentChat`

## 4. 约束与边界
- Enter 发送判定必须兼顾 IME 组合输入，避免误触发送。
- 按钮发送行为不变，保持兜底路径。
- Prompt Ops 面板不受本专题影响。
- 本轮不更改协议与会话聚合逻辑。

## 5. 设计演进计划
- 先补键盘判定与发送触发入口。
- 再完善 IME 兼容条件和单元测试。
- 最后更新手册并收口 Chat 输入体验。
