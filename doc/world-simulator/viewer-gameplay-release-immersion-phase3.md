# Viewer 发行体验改造（第三阶段：情绪闭环与世界活性）

## 目标
- 在第二阶段“反馈/引导/HUD”基础上，继续提升可玩性与情绪价值：
  - 增加里程碑成就反馈，强化“我做成了什么”的即时满足。
  - 增加 Agent 事件气泡反馈，强化“世界在说话”的活性体验。
  - 保持 Player 低噪声设计，不回退为技术工具面板。

## 范围
- `crates/agent_world_viewer` UI 侧改造：
  - Player 成就弹层（解锁里程碑、分级展示、自动淡出）。
  - Player Agent 事件气泡层（增量事件驱动，短句高频反馈）。
  - 现有 Player 体验层（HUD/引导/toast）整合为统一渲染层。
- 中英文文案保持一致并可切换。

## 非目标
- 不修改仿真核心协议与 `agent_world` 核心逻辑。
- 不改 `third_party` 目录。
- 不引入大型美术资源包（模型/贴图资产）。

## 接口/数据

### 本地 UI 状态
- 新增/扩展 Player 本地状态：
  - 成就解锁集合与成就弹层队列。
  - Agent 气泡队列与增量事件游标。
- 保持 `ViewerExperienceMode` 分流：
  - `player`：启用成就与气泡反馈。
  - `director`：保留调试导向行为，不强制展示。

### 触发策略
- 成就触发：连接成功、首次事件、首次选中目标、首次展开面板等关键里程碑。
- 气泡触发：`ViewerState.events` 增量事件（移动、采集、建造、拒绝等）。
- 防噪音约束：
  - 首帧不回放历史事件。
  - 队列长度受限并自动淡出。

## 里程碑
- M1：第三阶段建档完成（设计 + 项管）。
- M2：Player 成就反馈落地并具备单测覆盖。
- M3：Agent 事件气泡反馈落地并具备单测覆盖。
- M4：回归测试 + Web 闭环验收 + 文档收口。

## 风险
- 过度反馈导致视觉噪音上升。
  - 对策：严格队列上限、短时淡出、仅展示高价值短句。
- 多层浮层遮挡主场景。
  - 对策：固定边缘布局、层级错位、尺寸约束。
- 文案与事件类型映射不准确。
  - 对策：优先使用已验证事件语义，并补齐映射单测。

## 里程碑状态（2026-02-23）
- M1：完成（第三阶段设计/项目文档已建立）。
- M2：完成（Player 成就反馈已落地，含防重入与淡出单测）。
- M3：完成（Agent 事件气泡已落地，含增量游标/队列管理单测）。
- M4：完成（回归测试 + Web 闭环验收 + 文档收口已完成）。

## 验收结论
- 代码回归：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer sync_player_achievements_ -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer sync_agent_chatter_bubbles_ -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer feedback_ -- --nocapture`
- Web 闭环（Playwright）：
  - `window.__AW_TEST__.getState()` 可访问，`connectionStatus=connected`；
  - 页面基础语义正常（`title=Agent World Viewer (Web)`，`canvasCount=1`，`hasTestApi=true`）；
  - `Tab` 面板开关可用，Player HUD/引导层与右侧面板切换可观察。
- 验收截图：
  - `output/playwright/viewer/viewer-web-vri3p3-player-overlays-hidden.png`
  - `output/playwright/viewer/viewer-web-vri3p3-player-overlays-panel-open.png`
  - `output/playwright/viewer/viewer-web-vri3p3-player-overlays-panel-closed.png`
  - `output/playwright/viewer/viewer-web-vri3p3-player-overlays-selected-achievement-open-panel.png`
- 观察说明：
  - 本次 Web 运行中 `tick/eventCount` 维持 0，未在同次页面会话直接触发事件气泡可视样例；该链路由单测覆盖（`sync_agent_chatter_bubbles_*`）并确认增量事件到气泡队列逻辑正常。
- 阶段结论：
  - 第三阶段已完成“成就反馈 + Agent 世界反馈”能力建设，Player 体验层形成 HUD/引导/成就/气泡的统一渲染闭环，且未回退为技术面板主导形态。
