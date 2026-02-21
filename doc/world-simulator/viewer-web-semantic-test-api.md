# Viewer Web 语义化测试 API（Phase 9 发行验收支撑）

## 目标
- 为 Web 端 `agent_world_viewer` 注入一套稳定的语义化测试 API，降低 Playwright 对像素坐标点击的依赖。
- 复用现有 `viewer_automation` 步骤执行器，统一测试动作语义（`mode/focus/select/zoom/orbit/wait`）。
- 在不暴露生产攻击面的前提下，提供测试模式专用入口：`window.__AW_TEST__`。

## 范围

### In Scope
- 新增 `window.__AW_TEST__` 最小 API：
  - `runSteps(steps: string)`
  - `setMode(mode: "2d" | "3d")`
  - `focus(target: string)`
  - `select(target: string)`
  - `sendControl(action: "play" | "pause" | "seek", payload?: object)`
  - `getState()`
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

## 接口 / 数据

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

### 状态快照
- Bevy 每帧发布到可读快照：
  - `connection_status`
  - `tick`
  - `selected_kind` / `selected_id`
  - `camera_mode` / `camera_radius` / `camera_ortho_scale`
  - `error_count` / `last_error`
  - `event_count` / `trace_count`

## 里程碑
- WTA-0：设计/项目管理文档建档。
- WTA-1：`viewer_automation` 支持运行时步骤入队。
- WTA-2：`web_test_api`（wasm）桥接层实现与 `window.__AW_TEST__` 注入。
- WTA-3：`app_bootstrap` 接入命令消费与状态发布系统。
- WTA-4：单测与回归验证（`agent_world_viewer`）。
- WTA-5：文档状态与 devlog 收口。
- WTA-6：`testing-manual.md` S6 示例迁移到语义 API。
- WTA-7：`getState()` 扩展相机语义字段，支撑 zoom 可验证门禁。

## 风险
- Web 线程与 Bevy 主线程并发风险：
  - 缓解：只允许 JS 入队，不允许直接改资源。
- 测试 API 暴露到生产风险：
  - 缓解：仅在测试模式启用，默认 release 不开放。
- 语义命令解析失败导致测试不稳定：
  - 缓解：复用已有 `viewer_automation` 解析规则，并对非法输入忽略/记录。
