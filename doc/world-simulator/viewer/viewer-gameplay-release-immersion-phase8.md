# Viewer 发行体验改造（第八阶段：信息分层减负与焦点收敛）

## 目标
- 在保持“可随时指挥 Agent”前提下，进一步降低 Player 首屏的信息密度，避免左侧引导卡片重复表达同一目标。
- 让玩家在隐藏态与展开态都能快速识别“当前应该做什么”，减少任务文案和引导文案的视觉竞争。
- 在面板展开时压缩任务 HUD 体量，确保视线焦点优先回到世界场景和指挥面板。

## 范围
- `crates/agent_world_viewer` Player 模式 UI 减负改造：
  - 调整“下一步目标提示卡”与“新手引导卡”的共存策略，消除重复提示。
  - 调整任务 HUD 在不同面板状态下的布局锚点与信息密度（展开态更紧凑）。
  - 保留并强化“任务目标 + 指挥入口 + 世界反馈”三要素的可达性。
- 不改动 Director 模式行为。

## 非目标
- 不改动仿真协议、事件语义和后端链路。
- 不改动 `third_party` 代码。
- 不在本阶段引入新的 UI 框架或复杂动画系统。

## 接口/数据

### 复用状态
- `PlayerOnboardingState`：用于判定当前步骤是否仍需展示新手引导。
- `RightPanelLayoutState`：用于判定面板隐藏/展开，并驱动 HUD 密度切换。
- `PlayerGuideProgressSnapshot` 与 `PlayerGuideStep`：用于任务目标与引导步骤一致化。

### 新增/调整行为
- 目标提示卡显示策略：
  - 仅在“面板隐藏且当前步骤引导卡已收起”时显示，避免与引导卡重复。
- 任务 HUD 自适应策略：
  - 面板隐藏且引导卡显示时，下移任务 HUD 锚点避免叠压。
  - 面板展开时启用紧凑模式（保留核心目标与进度，降低次要文案占比）。

## 里程碑
- M1：第八阶段建档完成（设计 + 项管）。
- M2：完成目标提示卡与引导卡去重策略，并有单测覆盖。
- M3：完成任务 HUD 锚点与紧凑模式改造，并有单测覆盖。
- M4：完成回归测试、Web 闭环验证与文档收口。

## 风险
- 风险 1：提示层收敛后，部分玩家可能遗漏“下一步目标”。
  - 对策：保留顶部 compact HUD 的 objective 字段，并在引导卡收起后恢复底部目标提示卡。
- 风险 2：展开态任务 HUD 过度压缩导致可读性下降。
  - 对策：仅压缩冗余文案与奖励块，保留目标标题、进度条与关键动作按钮。
- 风险 3：新增状态分支导致行为不可预测。
  - 对策：将显示策略抽为纯函数并增加定向单测，保证规则可验证。

## 验收结果（VRI8P3）
- 回归结果：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（325 tests）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown` 通过。
- Web 闭环（Playwright，按 `testing-manual.md` S6）：
  - 使用 `?test_api=1` 访问 viewer，`window.__AW_TEST__` 可用，状态采样为已连接且 tick 正常推进。
  - 完成隐藏态、聚焦态、面板展开态（紧凑任务 HUD）与再次收起态截图采样。
  - Console 汇总：`Total messages: 12 (Errors: 0, Warnings: 2)`，无新增功能错误。
- 验收产物：
  - `output/playwright/viewer/phase8/phase8-hidden-default.png`
  - `output/playwright/viewer/phase8/phase8-hidden-focused.png`
  - `output/playwright/viewer/phase8/phase8-panel-open-compact.png`
  - `output/playwright/viewer/phase8/phase8-panel-hidden-after-toggle.png`
  - `.playwright-cli/console-2026-02-23T12-40-54-234Z.log`
- 结论：
  - 目标提示与引导提示的重复展示已消除。
  - 任务 HUD 在展开态的体量显著收敛，隐藏态与引导卡不再发生叠压。
  - “世界优先 + 可随时指挥 Agent”的布局目标在 phase8 达成。
