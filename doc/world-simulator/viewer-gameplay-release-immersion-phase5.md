# Viewer 发行体验改造（第五阶段：沉浸式布局与新手闭环）

## 目标
- 进一步把 Viewer 的 Player 体验从“工具操作界面”推进到“可发行游戏界面”：
  - 主场景占比稳定在高优先级（默认感知 85%+），右侧信息面板从“默认主视图”降级为“按需呼出辅助层”。
  - 新手进入后在 30 秒内明确“当前一步目标 -> 可执行动作 -> 反馈结果”的闭环路径。
  - 降低技术信息噪声，保留可调试能力但默认不干扰 Player 体验。

## 范围
- `crates/agent_world_viewer` Player 模式 UI 重构：
  - 右侧面板隐藏态升级为边缘呼出式入口（Edge Drawer Entry），增强可发现性与低干扰并存。
  - Player 模式右侧面板宽度预算收紧，避免打开后再次占据过多视野。
  - 新手引导卡与目标卡联动为“任务闭环提示”（阶段描述 + 快捷入口 + 状态回显）。
  - 拆分 `egui_right_panel_player_experience.rs`，确保单文件长度约束可持续。
- 保持中英文文案一致可切换。

## 非目标
- 不改动 `agent_world` 仿真协议与核心规则。
- 不改 `third_party`。
- 不引入大型美术资产包或外部 UI 框架。

## 接口/数据

### 本地状态扩展
- 扩展 Player 引导本地状态（仅 UI 层）：
  - 引导步骤可见性与最近交互时刻；
  - 入口卡“呼吸强调/边缘吸附”参数；
  - 任务闭环阶段文本与操作建议。

### 布局策略
- Player 模式：
  - 右侧面板默认隐藏；
  - 隐藏态显示右上角低侵入入口卡 + 边缘细条提示；
  - 打开态最大宽度受更严格预算限制（保证主画面占比）。
- Director 模式：
  - 保持现有调试导向布局，不强制启用 Player 限制。

### 触发策略
- 边缘入口提示：Player + 右侧面板隐藏时持续存在，低频脉冲。
- 引导闭环提示：按 `Connect -> OpenPanel -> Select -> Explore` 步骤推进；步骤切换触发轻动效。
- 快捷操作：保留 `Tab` 面板显隐；入口卡按钮与快捷键语义一致。

## 里程碑
- M1：第五阶段建档（设计+项管）完成。
- M2：Player 右侧面板结构重构完成（边缘呼出 + 宽度预算约束）。
- M3：新手任务闭环提示完善，并完成 `player_experience` 拆分（Rust 文件 <=1200 行）。
- M4：回归 + Web 闭环验收 + 文档收口。

## 风险
- 入口提示过弱导致新手找不到面板入口。
  - 对策：隐藏态保留“模式说明 + 快捷键 + 主按钮”三要素，且边缘提示持续可见。
- 面板宽度限制过严影响深度操作。
  - 对策：仅在 Player 模式限制；Director 维持原能力。
- 文案与交互状态不一致导致误导。
  - 对策：将步骤状态统一由 `resolve_player_guide_step` 驱动，减少分叉逻辑。

## 验收与结论
- 里程碑完成情况：
  - M1/M2/M3/M4 全部完成。
- 回归结果：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（309/309）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown` 通过。
- Web 闭环（S6）结果：
  - `window.__AW_TEST__` 可访问，`getState()` 返回 `tick/connectionStatus`。
  - 语义动作链可用：`runSteps(...)` + `sendControl("pause")` + `Tab` 面板切换。
  - 控制台 `Errors: 0`（统计 warning 1 条）。
  - 验收截图：
    - `output/playwright/viewer/viewer-web-vri5p3-player-edge-hidden.png`
    - `output/playwright/viewer/viewer-web-vri5p3-player-edge-open.png`
- 视觉观察（基于实际 Web 画面）：
  - 隐藏态下新增右侧边缘呼出提示，面板入口更易发现且不压主场景。
  - 引导卡/目标卡显示 4 步闭环进度（`✓ / ▶ / ·` + `x/4`），新手“当前该做什么”更明确。
  - Player 打开面板时宽度受预算约束，主场景可见面积相较之前更稳定。
