# Viewer 发行体验改造（第六阶段：任务驱动与电影化首屏）

## 目标
- 继续将 Viewer 的 Player 体验从“可操作”推进到“可发行游戏体验”：
  - 首次进入 10 秒内建立世界氛围与行动动机，避免“工具界面直出”。
  - 将新手引导升级为任务驱动闭环，让玩家持续知道“下一步做什么”。
  - 强化关键行为的视觉奖励反馈（完成、发现、推进），提升情绪价值。
  - 继续保持主场景优先，右侧面板作为可选辅助层。

## 范围
- `crates/agent_world_viewer` 的 Player 模式 UI 增强：
  - 电影化开场层（连接后短时叙事覆盖 + 自动淡出）。
  - 任务追踪 HUD（主任务标题、步骤进度、当前动作提示）。
  - 奖励反馈升级（任务推进提示、完成态强化文案与颜色层级）。
  - 小地图卡片（位置缩略可视化 + 当前选中高亮），提高方向感。
  - Player 模式动效统一（低频、轻量，不遮挡核心交互）。
- 保持 Director 模式调试导向布局不变。
- 保持中英文文案可切换。

## 非目标
- 不改动 `agent_world` 核心仿真协议、规则模块与事件定义。
- 不改 `third_party`。
- 不引入大型美术资产包或外部 UI 框架。

## 接口/数据

### 输入数据（复用）
- `ViewerState.status/snapshot/events/metrics`：用于连接态、世界位置、事件推进与 Tick 进度。
- `ViewerSelection.current`：用于任务步骤判定与小地图选中高亮。
- `RightPanelLayoutState.panel_hidden`：用于任务闭环步骤（OpenPanel）与交互提示联动。

### UI 层新增能力
- 电影化开场快照计算：
  - 根据连接态与世界 Tick 计算开场显示窗口与透明度曲线（淡入/停留/淡出）。
- 任务闭环增强快照：
  - 复用 `Connect -> OpenPanel -> Select -> Explore` 步骤，输出“完成数、当前步骤、下一动作文案”。
- 小地图快照：
  - 从 `snapshot.model.locations` 归一化世界平面坐标并渲染点位；
  - 根据 `selection` 计算高亮点。

### 约束
- 所有新增交互与视觉能力须可在 Web 闭环（S6）中验证。
- 默认视觉策略遵循“低遮挡、低频动效、强可读”原则。

## 里程碑
- M1：第六阶段建档完成（设计 + 项管）。
- M2：电影化开场层落地并具备单测覆盖。
- M3：任务驱动 HUD 与奖励推进反馈落地并具备单测覆盖。
- M4：小地图卡片与任务/选中联动落地并具备单测覆盖。
- M5：回归测试 + Web 闭环验收 + 文档收口。

## 验收与结论
- 验收日期：2026-02-23（CST）。
- 回归结果（S5）：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer`：通过（317 passed, 0 failed）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown`：通过。
- Web 闭环结果（S6）：
  - 语义 API：`window.__AW_TEST__` 可用，`getState()` 返回结构完整（`tick/connectionStatus`）。
  - 交互链路：`runSteps("mode=3d;focus=first_location;zoom=0.85;select=first_agent;wait=0.3")`、`sendControl("pause")`、`Tab` 折叠/展开均可执行。
  - 控制台：`Errors: 0`（Warnings: 4，均为 WebGPU/AudioContext 平台告警）。
- 产物：
  - `output/playwright/viewer/phase6/phase6-cinematic.png`
  - `output/playwright/viewer/phase6/phase6-panel-open.png`
  - `output/playwright/viewer/phase6/phase6-panel-hidden.png`
  - `output/playwright/viewer/phase6/phase6-mission-minimap.png`
  - `output/playwright/viewer/phase6/state-final.txt`
  - `output/playwright/viewer/phase6/console.log`
- 结论：
  - 第六阶段目标已完成，Player 模式形成“电影化开场 -> 任务驱动 -> 奖励反馈 -> 小地图导向”的连续体验链路；
  - 在真实 Web 环境中验证通过，未发现阻断发行体验的回归问题。

## 风险
- 新增浮层过多导致遮挡主场景。
  - 对策：所有新增卡片限定宽度与边缘锚点；开场层自动短时淡出。
- 任务文案与实际状态不同步。
  - 对策：统一以现有 `PlayerGuideStep` 与 progress snapshot 驱动，不引入第二套状态机。
- 小地图密集点位可读性下降。
  - 对策：限制尺寸与符号层级（普通点/选中点），优先显示选中目标。
