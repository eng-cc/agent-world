# 客户端启动器 Web wasm 时间兼容与闭环修复（2026-03-04）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: `oasis7_client_launcher` Web 端在浏览器启动后触发 `time not implemented on this platform` panic，导致 egui wasm UI 初始化失败，无法完成启动/停止闭环。
- Proposed Solution: 为 launcher wasm 路径替换为 Web 兼容的计时实现，并将 agent-browser 闭环（open/snapshot/console/state）纳入验收，阻断同类回归。
- Success Criteria:
  - SC-1: 浏览器打开 launcher Web 页面后不再出现 `time not implemented on this platform` 或 `RuntimeError: unreachable`。
  - SC-2: launcher Web 页面可稳定渲染，且 `/api/state` 轮询可持续运行。
  - SC-3: `agent-browser --headed` 闭环可完成页面加载、console 抽样与截图采证。
  - SC-4: 相关改动保持 native 启动器行为不回归。
  - SC-5: `self_guided` 路径在 wasm 目标不再依赖 `std::time::SystemTime`，并可通过浏览器控制台回归校验。

## 2. User Experience & Functionality
- User Personas:
  - 启动器开发者：需要 native/wasm 同层代码在浏览器稳定运行。
  - Web 闭环执行者：需要 agent-browser 验收可复现且可审计。
- User Scenarios & Frequency:
  - 每次 launcher Web 端基础能力改动后执行 1 次 smoke（高频）。
  - 发布前执行至少 1 次 Web 闭环采证（每次发布）。
- User Stories:
  - As a 启动器开发者, I want wasm path to use web-compatible time primitives, so that launcher UI does not panic on browser startup.
  - As a 测试执行者, I want agent-browser evidence to include console/state/screenshot, so that regressions are quickly diagnosable.
- Critical User Flows:
  1. Flow-LAUNCHER-WASM-TIME-001（页面加载）:
     `启动 oasis7_web_launcher -> 浏览器打开 / -> wasm 初始化 -> egui UI 可交互`
  2. Flow-LAUNCHER-WASM-TIME-002（状态轮询）:
     `UI 定时 GET /api/state -> 页面状态更新 -> 无 panic/阻塞`
  3. Flow-LAUNCHER-WASM-TIME-003（闭环采证）:
     `agent-browser open --headed -> snapshot -> console error -> screenshot -> 产物归档`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| wasm 轮询计时兼容 | `last_web_poll_at`、`WEB_POLL_INTERVAL_MS` | 定时触发状态查询，避免并发请求堆积 | `idle -> polling -> synced` | 下一次轮询基于单调时钟差值 | 浏览器会话可读 |
| Web 初始化稳定性 | `wasm startup`、`console error` | 页面加载后进入 launcher egui 主界面 | `boot -> interactive` | 初始化失败立即记录错误并阻断验收 | 测试/运维可见 |
| agent-browser 证据 | `snapshot/console/screenshot/state` | 执行固定命令序列采集证据 | `running -> evidence_ready` | 证据目录固定 `output/playwright/` | 执行者可写 |
- Acceptance Criteria:
  - AC-1: `oasis7_client_launcher` wasm 路径使用 Web 兼容时间实现，不再调用不支持的平台时间 API。
  - AC-2: `env -u RUSTC_WRAPPER cargo check -p oasis7_client_launcher --target wasm32-unknown-unknown` 通过。
  - AC-3: `agent-browser --headed` 打开 launcher 页面后，`console error` 中不包含 `time not implemented on this platform`。
  - AC-4: 闭环证据包含至少 1 份 snapshot、1 份 console、1 张 screenshot。
  - AC-5: `oasis7_web_launcher` API（至少 `/api/state`）在闭环期间保持可用。
  - AC-6: `self_guided::current_unix_ms` 在 wasm 目标使用 Web 兼容实现，避免触发 `time not implemented` panic。
- Non-Goals:
  - 不在本轮扩展新的 launcher Web 功能模块。
  - 不在本轮重构 `oasis7_web_launcher` 进程编排协议。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 模型能力需求。

## 4. Technical Specifications
- Architecture Overview:
  - `oasis7_client_launcher` 在 `cfg(target_arch = "wasm32")` 路径使用 Web 兼容时间类型。
  - `oasis7_web_launcher` 保持 API 与静态托管职责；验证重点在 wasm UI 初始化与轮询稳定性。
  - agent-browser CLI 作为 Web 闭环执行器，输出标准证据。
- Integration Points:
  - `crates/oasis7_client_launcher/src/main.rs`
  - `crates/oasis7_client_launcher/src/app_process_web.rs`
  - `crates/oasis7_client_launcher/Cargo.toml`
  - `crates/oasis7/src/bin/oasis7_web_launcher.rs`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 浏览器平台不支持 `std` 时间 API：必须使用 Web 兼容计时实现，避免 panic。
  - `main.rs` 已切换 Web 兼容计时，但 `self_guided` 子模块遗漏 `SystemTime::now`：应在同一专题内补齐，防止回归。
  - API 临时失败：UI 回退 `query_failed` 并记录日志，但进程不崩溃。
  - 轮询请求重叠：通过 `web_request_inflight` 防抖，避免并发风暴。
- Non-Functional Requirements:
  - NFR-1: launcher Web 页首屏初始化无 panic，`console error` 不出现时间平台不支持错误。
  - NFR-2: `WEB_POLL_INTERVAL_MS` 轮询下请求不会无界堆积。
  - NFR-3: agent-browser 闭环证据固定归档在 `output/playwright/launcher-web-closure-*`。
  - NFR-4: 不影响 native 启动器构建与单测通过。
  - NFR-5: Web 启动器 smoke 在 `start/stop` 闭环后控制台仍保持 `time not implemented` 零命中。
- Security & Privacy:
  - 闭环采证不得输出敏感配置（如 key/token）；仅记录必要诊断信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1: 建模与任务拆解（文档先行）。
  - M2: wasm 时间兼容修复与定向编译验证。
  - M3: agent-browser 闭环复测并归档证据。
  - M4: 回归补丁：覆盖 `self_guided` 时间调用点并执行启动器 Web 控制台启停闭环复验。
- Technical Risks:
  - 风险-1: wasm 兼容修复遗漏其他时间调用点，导致间歇性崩溃。
  - 风险-2: 修复后 native 路径行为意外变化。
  - 风险-3: 闭环脚本仅校验 API 状态而忽略 console，可能把前端 panic 误判为通过。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-013 -> TASK-WORLD_SIMULATOR-029/030/097 -> `test_tier_required`。
- Decision Log:
  - DEC-LAUNCHER-WASM-TIME-001: 采用“wasm 路径切换 Web 兼容计时类型 + `agent-browser --headed` 闭环验证”方案，而非仅在文档中标记已知问题。理由：该问题直接阻断 Web 可用性，必须以代码修复和自动化证据闭环收敛。
  - DEC-LAUNCHER-WASM-TIME-002: 对同一 PRD 补充回归修复子任务（`self_guided`），不新建主题文档。理由：问题属于既有时间兼容专题的遗漏点，继续沿用同一 PRD-ID 可保持追溯连续性。
  - DEC-LAUNCHER-WASM-TIME-003: `self_guided` 时间函数在 wasm 目标统一使用 `web_time::SystemTime`。理由：保持与既有 Web 兼容依赖一致，避免引入额外时间库并消除 `std::time` 平台 panic。
