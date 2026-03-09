# Viewer Web 语义化测试 API（Phase 9 发行验收支撑）

审计轮次: 5
- 对应项目管理文档: doc/world-simulator/viewer/viewer-web-semantic-test-api.prd.project.md

## 1. Executive Summary
- 为 Web 端 `agent_world_viewer` 注入一套稳定的语义化测试 API，降低 Playwright 对像素坐标点击的依赖。
- 复用现有 `viewer_automation` 步骤执行器，统一测试动作语义（`mode/focus/select/zoom/orbit/wait`）。
- 在不暴露生产攻击面的前提下，提供测试模式专用入口：`window.__AW_TEST__`。
- 补齐 round-1 目标：对齐人类高频操作中的“面板显隐 / 模块显隐 / 选中聚焦 / 材质预览切换”四类语义动作，减少脚本对键盘事件与像素点击的耦合。
- 补齐 round-2 目标：对齐“顶部区显隐 / 语言切换 / 玩家布局预设（Mission/Command/Intel）”三类 UI 状态动作，进一步降低脚本对界面点击路径的依赖。
- 补齐 round-3 目标：对齐“聊天消息发送 / Prompt 三字段覆盖提交”两类高频输入动作，减少脚本对 Chat 面板逐控件输入的依赖。
- 补齐 round-4 目标：对齐“时间轴 seek / 标记过滤 / 标记跳点”三类时间轴操作，减少脚本对 timeline 区块逐按钮点击的依赖。

## 2. User Experience & Functionality

### In Scope
- 新增 `window.__AW_TEST__` 最小 API：
  - `runSteps(steps: string)`
  - `setMode(mode: "2d" | "3d")`
  - `focus(target: string)`
  - `select(target: string)`
  - `sendControl(action: "play" | "pause" | "step" | "seek", payload?: object)`
  - `getState()`
- `runSteps(steps: string)` round-1 补齐语义：
  - `panel=<show|hide|toggle>`
  - `module=<controls|overview|chat|overlay|diagnosis|event_link|timeline|details>:<show|hide|toggle>`
  - `focus=selection`（或 `focus_selection=current`）
  - `material_variant=<next|cycle>`
- `runSteps(steps: string)` round-2 补齐语义：
  - `top_panel=<show|hide|toggle>`
  - `locale=<zh|en|toggle>`（或 `language=<zh|en|toggle>`）
  - `layout=<mission|command|intel>`
- `runSteps(steps: string)` round-3 补齐语义：
  - `chat=<agent_id>|<message>`（`message` 支持 `%xx` 文本解码）
  - `prompt_system=<agent_id>|<text|clear>`
  - `prompt_short=<agent_id>|<text|clear>`
  - `prompt_long=<agent_id>|<text|clear>`
- `runSteps(steps: string)` round-4 补齐语义：
  - `timeline_seek=<tick>`
  - `timeline_filter=<err|llm|peak>:<show|hide|toggle>`
  - `timeline_jump=<err|llm|peak>`
- 通过命令队列将 JS 调用转为主线程逐帧消费，避免并发修改 Bevy 资源。
- 通过 `getState()` 返回闭环测试关键状态：
  - 连接状态
  - 当前 tick
  - 当前选中对象
  - 相机状态（`cameraMode` / `cameraRadius` / `cameraOrthoScale`）
  - 错误计数与最近错误
  - 事件/trace 计数
- 接入 `app_bootstrap` 生命周期（startup 注册、update 消费与状态发布）。

### Out of Scope
- 不改 Viewer 协议（`ViewerRequest/ViewerResponse` 语义不变）。
- 不扩展到通用远程控制协议（仅用于 Web UI 测试辅助）。
- 不在本阶段引入完整 Playwright E2E 套件重写（先提供 API 与最小回归）。
- 不在本轮覆盖文本输入、鼠标拖拽轨迹细节、IME 组合输入等低层“逐事件复刻”能力。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications

### 测试入口
- JS 全局对象：`window.__AW_TEST__`（仅测试模式启用）。
- 运行时开关：
  - `cfg(debug_assertions)` 下默认启用；
  - 或 URL query 包含 `?test_api=1`（release 场景）。

### 命令队列
- JS -> queue -> Bevy Update 消费：
  - `RunSteps(String)`
  - `SetMode(ViewerCameraMode)`
  - `Focus(ViewerAutomationTarget)`
  - `Select(ViewerAutomationTarget)`
  - `SendControl(ViewerControl)`
- `RunSteps` round-1 扩展步骤类型：
  - `PanelVisibility`
  - `PanelModuleVisibility`
  - `FocusSelection`
  - `CycleMaterialVariant`
- `RunSteps` round-2 扩展步骤类型：
  - `TopPanelVisibility`
  - `SetLocale`
  - `PlayerLayoutPreset`
- `RunSteps` round-3 扩展步骤类型：
  - `SendAgentChat`
  - `ApplyPromptOverride`
- `RunSteps` round-4 扩展步骤类型：
  - `TimelineSeek`
  - `TimelineMarkFilter`
  - `TimelineMarkJump`

### 状态快照
- Bevy 每帧发布到可读快照：
  - `connection_status`
  - `tick`
  - `selected_kind` / `selected_id`
  - `camera_mode` / `camera_radius` / `camera_ortho_scale`
  - `error_count` / `last_error`
  - `event_count` / `trace_count`

## 5. Risks & Roadmap
- WTA-0：设计/项目管理文档建档。
- WTA-1：`viewer_automation` 支持运行时步骤入队。
- WTA-2：`web_test_api`（wasm）桥接层实现与 `window.__AW_TEST__` 注入。
- WTA-3：`app_bootstrap` 接入命令消费与状态发布系统。
- WTA-4：单测与回归验证（`agent_world_viewer`）。
- WTA-5：文档状态与 devlog 收口。
- WTA-6：`testing-manual.md` S6 示例迁移到语义 API。
- WTA-7：`getState()` 扩展相机语义字段，支撑 zoom 可验证门禁。
- WTA-8（已完成）：round-1 补齐需求建模与任务拆解（文档任务）。
- WTA-9（已完成）：落地 `viewer_automation` round-1 语义步骤（panel/module/focus_selection/material_variant）。
- WTA-10（已完成）：执行 round-1 定向回归并完成文档收口。
- WTA-11：round-2 补齐需求建模与任务拆解（文档任务）。
- WTA-12（已完成）：落地 `viewer_automation` round-2 语义步骤（top_panel/locale/layout）并补齐解析/映射测试。
- WTA-13（已完成）：执行 round-2 定向回归、更新手册示例与文档状态收口。
- WTA-14（已完成）：round-3 补齐需求建模与任务拆解（聊天发送/Prompt 覆盖提交语义）。
- WTA-15（已完成）：落地 `viewer_automation` round-3 语义步骤（chat/prompt）并补齐解析/映射测试。
- WTA-16（已完成）：执行 round-3 定向回归、更新手册示例与文档状态收口。
- WTA-17（已完成）：round-4 补齐需求建模与任务拆解（timeline seek/filter/jump 语义）。
- WTA-18（已完成）：落地 `viewer_automation + web_test_api` round-4 语义步骤（timeline + sendControl.seek）并补齐定向测试。
- WTA-19（已完成）：执行 round-4 定向回归、更新手册示例与文档状态收口。

### Technical Risks
- Web 线程与 Bevy 主线程并发风险：
  - 缓解：只允许 JS 入队，不允许直接改资源。
- 测试 API 暴露到生产风险：
  - 缓解：仅在测试模式启用，默认 release 不开放。
- 语义命令解析失败导致测试不稳定：
  - 缓解：复用已有 `viewer_automation` 解析规则，并对非法输入忽略/记录。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WTA-R1-001 -> WTA-8 -> `test_tier_required`（文档存在性与条目一致性检查）
  - PRD-WTA-R1-002 -> WTA-9 -> `test_tier_required`（`agent_world_viewer` 定向单测）
  - PRD-WTA-R1-003 -> WTA-10 -> `test_tier_required`（`cargo check` + 文档回写追溯）
  - PRD-WTA-R2-001 -> WTA-11 -> `test_tier_required`（文档存在性与条目一致性检查）
  - PRD-WTA-R2-002 -> WTA-12 -> `test_tier_required`（`agent_world_viewer` 定向单测）
  - PRD-WTA-R2-003 -> WTA-13 -> `test_tier_required`（`cargo check` + 手册示例可达 + 文档回写追溯）
  - PRD-WTA-R3-001 -> WTA-14 -> `test_tier_required`（文档存在性与条目一致性检查）
  - PRD-WTA-R3-002 -> WTA-15 -> `test_tier_required`（`agent_world_viewer` 定向单测）
  - PRD-WTA-R3-003 -> WTA-16 -> `test_tier_required`（`cargo check` + 手册示例可达 + 文档回写追溯）
  - PRD-WTA-R4-001 -> WTA-17 -> `test_tier_required`（文档存在性与条目一致性检查）
  - PRD-WTA-R4-002 -> WTA-18 -> `test_tier_required`（`agent_world_viewer` 定向单测）
  - PRD-WTA-R4-003 -> WTA-19 -> `test_tier_required`（`cargo check` + 手册示例可达 + 文档回写追溯）
- Decision Log:
  - 采用 `runSteps` 语义扩展而不是新增大量 JS API 方法，避免 `web_test_api.rs` 持续膨胀并贴近单文件上限。
  - round-1 优先补齐“操作语义”而非“原始输入事件回放”，以降低 Web 闭环脚本脆弱性。
  - round-2 继续优先覆盖“UI 状态切换语义”（top/locale/layout），暂不进入聊天输入文本等高自由度 payload 场景。
  - round-3 采用“结构化参数 + `%xx` 文本解码”承载 chat/prompt 文本输入，保持 `runSteps` 单入口而不新增 `sendChat/applyPrompt` 等方法。
  - round-4 采用“runSteps 承载 timeline 视图语义 + sendControl 承载 seek 控制语义”的双层方案，兼容回放与 live 模式差异（live 下 seek 明确无效回执）。
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
