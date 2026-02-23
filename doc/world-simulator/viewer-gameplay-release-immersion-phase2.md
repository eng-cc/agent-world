# Viewer 发行体验改造（第二阶段：沉浸与引导）

## 目标
- 在已完成的 Player 默认降噪基础上，继续提升“可玩感”：
  - 关键动作可感知（情绪反馈）
  - 首次进入可快速上手（新手引导）
  - 主界面存在稳定且轻量的目标/HUD 引导
- 保持 Director 模式不受破坏，确保调试链路可用。

## 范围
- `crates/agent_world_viewer` 的 Web/UI 交互层：
  - 事件驱动情绪反馈（toast/提示）
  - Player 首屏引导层与“下一步目标”提示
  - Player 轻量 HUD（非技术细节优先）
- 文本与提示支持中英文切换。

## 非目标
- 本阶段不引入大型美术资源包（模型/贴图重做）。
- 本阶段不改动 `agent_world` 协议与仿真核心逻辑。
- 本阶段不改 `third_party` 目录。

## 接口/数据

### 运行时状态
- 新增 UI 本地状态（仅 viewer 内部）：
  - 反馈队列（最近事件生成 toast）
  - 引导可见性状态（Player 首屏引导可关闭）
- 现有 `ViewerExperienceMode` 继续作为行为分流入口：
  - `player`：启用反馈/引导/HUD
  - `director`：保持调试导向，不强制引导

### 触发策略
- 反馈来源：`ViewerState.events` 增量事件。
- 引导来源：连接状态、面板显隐、当前选择状态。
- HUD 来源：连接状态、tick、事件数、选择状态。

### 可用性约束
- 所有新增交互都必须可在 Web 闭环中验证（Playwright + `window.__AW_TEST__`）。
- 快捷键行为在聊天输入聚焦时不得干扰输入。

## 里程碑
- M1：建档完成（设计 + 项管）。
- M2：情绪反馈落地（事件 toast + 反馈色阶）。
- M3：新手引导落地（首屏提示 + 下一步目标）。
- M4：Player 轻量 HUD 落地并完成风格收敛。
- M5：测试回归与 Web 闭环验收，文档收口。

## 风险
- 反馈过载导致视觉噪音上升。
  - 对策：限制队列长度与单条展示时长，按严重性分层。
- 引导层遮挡主场景。
  - 对策：卡片化小体积布局，支持关闭/自动收起。
- 快捷键与 egui 焦点冲突。
  - 对策：显式检查聊天输入聚焦，避免吞键/误触。

## 里程碑状态（2026-02-23）
- M1：完成（已建档）。
- M2：完成（事件反馈 toast 已落地并有单测覆盖）。
- M3：完成（Player 新手引导与“下一步目标”提示已落地）。
- M4：完成（Player 紧凑 HUD 与入口卡风格收敛已落地）。
- M5：完成（回归测试 + Web 闭环验收 + 文档收口已完成）。

## 验收结论
- Web 闭环链路可用：`window.__AW_TEST__` 可访问，`connectionStatus=connected`，`canvasCount=1`。
- 关键视觉产物（Playwright 截图）：
  - `output/playwright/viewer/viewer-web-vri4-player-hud-hidden.png`
  - `output/playwright/viewer/viewer-web-vri4-player-hud-panel-open.png`
  - `output/playwright/viewer/viewer-web-vri4-player-hud-panel-closed-again.png`
- 阶段结论：Player 模式下已形成“反馈 + 引导 + HUD”的低噪声体验闭环，右侧技术面板回退为可选辅助路径。
