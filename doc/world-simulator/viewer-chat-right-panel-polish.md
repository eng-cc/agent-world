# Agent World Simulator：Viewer Chat 右侧收敛布局与闭环验收（设计文档）

## 目标
- 将 Chat UI 收敛为“最右侧单区块”的稳定布局，避免左右分裂导致的信息跳跃。
- 聊天记录固定放在右侧 Chat 区上方，Agent 选择与输入发送区域放在聊天记录下方。
- 通过 Web 闭环调试（Playwright）产出截图，验证布局与交互可用性。

## 范围

### 范围内
- 移除左侧 Chat History SidePanel。
- 右侧 Chat 区重排：
  - 上方：聊天记录（消息流）
  - 下方：Agent 选择、输入框、发送按钮、状态提示
- 保留现有会话聚合逻辑与消息气泡视觉，不改协议。
- 更新 3D 输入命中边界：仅避让右侧 panel（取消左侧历史栏避让）。
- 执行 Web 闭环并输出截图至 `output/playwright/viewer/`。

### 范围外
- 不改动 `AgentChat` 协议结构。
- 不新增后端会话持久化能力。
- 不改动 Prompt Ops 模式业务逻辑。

## 接口 / 数据
- 复用 `ViewerState.decision_traces[].llm_chat_messages[]` 作为聊天记录来源。
- UI 状态保留 `AgentChatDraftState`，继续维护当前 Agent/会话选择与输入草稿。
- 删除/下线左侧聊天历史宽度状态资源及其相机/拾取判定依赖。

## 里程碑
- M1：文档与任务拆解完成。
- M2：右侧 Chat 布局重排完成（记录在上、输入在下）。
- M3：3D 输入边界回归完成。
- M4：Web 闭环截图取证与文档收口完成。

## 风险
- 会话较长时聊天记录可视区域不足：通过滚动区高度约束与自动滚动策略缓解。
- 布局重排后输入焦点可能受影响：保留现有 IME 焦点桥接 ID 与聚焦信号。
- Web 闭环首次加载偶发空白：按既有流程排查 trunk 编译、端口、console 错误。
