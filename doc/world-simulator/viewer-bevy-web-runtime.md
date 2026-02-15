# Viewer Bevy 浏览器运行路径设计

## 目标
- 为 `agent_world_viewer` 增加一条可执行的浏览器运行路径（`wasm32-unknown-unknown`），让 Viewer 支持在浏览器中启动与渲染。
- 保持现有桌面运行方式不受影响：桌面仍支持在线 TCP 连接 `world_viewer_live`。
- 建立最小闭环：`wasm32` 编译可通过、可通过统一脚本启动本地 Web 调试服务、文档可直接指引使用。
- 将 Web 路径设为 Viewer 闭环默认路径，闭环证据统一走 `Playwright` 产物（截图 + console）。

## 范围
- 范围内：
  - 修复 `agent_world`/`agent_world_viewer` 在 `wasm32` 目标下的编译不兼容点。
  - 在 Viewer 中引入 Web 平台路径：浏览器端默认离线模式，不尝试 TCP 直连。
  - 新增基于 `trunk` 的浏览器启动入口（脚本 + `index.html`）。
  - 更新使用手册，补充 Web 运行步骤、Playwright 闭环步骤与限制说明。
- 范围外：
  - 不在本任务实现浏览器端与 `world_viewer_live` 的在线协议桥接（WebSocket/HTTP 中转）。
  - 不重构 Viewer 业务逻辑，不新增 Web 专属 UI 功能。
  - 不引入重型 Playwright test suite（仅保留 CLI 级最小闭环）。

## 接口 / 数据

### 1) 构建与运行入口
- 新增脚本：`scripts/run-viewer-web.sh`
  - 负责检查 `trunk` 与 `wasm32-unknown-unknown` target。
  - 在 `crates/agent_world_viewer` 下执行 `trunk serve`。
- 新增页面入口：`crates/agent_world_viewer/index.html`
  - 使用 trunk rust pipeline（`data-bin=agent_world_viewer`）构建并加载 wasm。

### 1.1) 默认闭环入口（策略约束）
- Web 启动：`./scripts/run-viewer-web.sh --address 127.0.0.1 --port 4173`
- Playwright CLI 闭环：
  - `open http://127.0.0.1:4173`
  - `snapshot`
  - `console`
  - `screenshot output/playwright/viewer/*.png`
- 最小验收：
  - 页面可加载（存在 `canvas`）
  - `console error = 0`
  - 至少 1 张截图产物

### 2) 平台行为约束
- Native（非 `wasm32`）：维持当前逻辑，支持 TCP 连接与 headless。
- Web（`wasm32`）：
  - 入口直接走 UI 模式。
  - 默认离线（`offline=true`），连接状态在 UI 中按离线模式展示。
  - 不编译 TCP 连接链路（`TcpStream` 相关代码通过 `cfg` 隔离）。

### 3) `agent_world` wasm 兼容点
- `OpenAiChatCompletionClient::build_http_client` 在 `wasm32` 下不设置 `timeout`，避免 `reqwest` wasm 平台 API 差异导致编译失败。

## 里程碑
- WBR1：输出设计文档与项目管理文档，并挂载总项目文档入口。
- WBR2：完成 `agent_world`/`agent_world_viewer` wasm 兼容改造并通过 `cargo check --target wasm32-unknown-unknown`。
- WBR3：完成 Web 启动入口（脚本 + `index.html`）与手册更新。
- WBR4：执行 `test_tier_required` 相关回归，更新项目状态与开发日志。
- WBR5：闭环策略收口（Web 默认、native fallback）并同步 AGENTS/手册/脚本文档。

## 风险
- 平台能力差异风险：浏览器端无法直接使用 TCP。
  - 缓解：本阶段明确 Web 路径仅支持离线 UI，在线能力后续通过桥接任务补齐。
- 依赖差异风险：`reqwest`/系统 API 在 wasm 下能力与 native 不同。
  - 缓解：针对已知不兼容 API 做 `cfg` 分支，并纳入 wasm 目标编译回归。
- 工具链风险：本地缺少 `trunk` 或 wasm target 导致无法启动。
  - 缓解：脚本中增加前置检查与明确报错提示。
- 规范漂移风险：历史文档仍指向 native 截图链路。
  - 缓解：以 `viewer-web-closure-testing-policy` 统一口径，定期在项目文档中校验。
