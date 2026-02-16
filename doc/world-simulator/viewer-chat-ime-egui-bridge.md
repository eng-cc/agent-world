# Agent World Simulator：Viewer Chat Web IME EGUI 输入桥接（设计文档）

## 目标
- 解决 Web Viewer 中“可粘贴中文但无法直接输入中文”的问题。
- 在 wasm 桌面浏览器下，为 EGUI 文本输入单独接入 IME 组合事件桥接。
- 保持现有 Chat 模块与协议（`AgentChat`）不变。

## 范围

### In Scope
- 在 `agent_world_viewer`（wasm32）中新增 EGUI 输入桥接模块。
- 通过隐藏 HTML input 捕获 `composition*` / `input` / 部分键盘事件，并转换为 `egui::Event`。
- 将转换后的事件注入 `bevy_egui` 主上下文（Primary EGUI Context）。
- 在 EGUI 文本编辑激活时自动 focus 输入桥接节点，非编辑态自动 blur。
- 回归验证：`cargo check`、关键测试、Web 闭环截图与 console 错误检查。

### Out of Scope
- Chat UI 布局与交互重构。
- Native 平台输入法行为改造。
- 上游 `bevy/bevy_egui` 源码修改。

## 接口 / 数据
- 不修改 Viewer 协议字段。
- 新增 wasm 内部桥接资源与事件通道：
  - DOM -> bridge event -> `egui::Event` -> `EguiInputEvent` -> EGUI 主上下文。
- 关键事件映射：
  - `compositionstart` -> `Ime(Enabled)`
  - `compositionupdate` -> `Ime(Preedit)`
  - `compositionend`/`input` -> `Text(...)` + `Ime(Disabled)`
  - `keydown/keyup`（部分按键）-> `Event::Key`

## 里程碑
- M1：设计文档与项目任务拆解。
- M2：实现 wasm EGUI IME bridge 与 app 启动接线。
- M3：执行回归验证与 Web 闭环取证。
- M4：文档回写、devlog、收口提交。

## 风险
- 浏览器差异可能导致 `compositionend` 与 `input` 时序不同。
- bridge 聚焦策略若处理不当，可能影响非文本快捷键。
- 事件重复注入风险需通过状态位与去重策略控制。
