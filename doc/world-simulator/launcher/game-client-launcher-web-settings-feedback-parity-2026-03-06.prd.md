# 客户端启动器 Web 设置/反馈功能对齐（2026-03-06）

审计轮次: 5
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-settings-feedback-parity-2026-03-06.prd.project.md`

## 1. Executive Summary
- Problem Statement: 启动器 native 与 web 已共用统一控制面，但 Web 端 `设置` 与 `反馈` 入口仍为禁用，占位实现导致跨端体验不一致。
- Proposed Solution: 在 wasm 端补齐设置中心可用 UI 与反馈提交流程，并在 `world_web_launcher` 新增反馈代理 API，使 Web 与 native 在核心功能上保持同层可用。
- Success Criteria:
  - SC-1: Web 端可打开设置窗口，支持编辑游戏/区块链配置并维护 LLM 连接参数。
  - SC-2: Web 端可打开反馈窗口并提交反馈到链运行时，不再显示“暂不支持”。
  - SC-3: 控制面新增 `/api/chain/feedback`，代理到 `world_chain_runtime` 的 `/v1/chain/feedback/submit`。
  - SC-4: `test_tier_required` 回归通过，且不引入 native 路径回归。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家（Web）：需要在浏览器中直接完成设置调整与问题反馈。
  - 运维/测试人员：需要在 headless 场景验证反馈提交与配置编辑闭环。
- User Scenarios & Frequency:
  - 日常联调：每次排查问题时，先调整设置，再提交反馈（高频）。
  - 发布前回归：每次发布前至少执行一次“设置打开 + 反馈提交”闭环（中频）。
- User Stories:
  - PRD-WORLD_SIMULATOR-021: As a Web 启动器玩家, I want settings and feedback entries to work in browser, so that I can complete the same control loop as native.
- Critical User Flows:
  1. Flow-LAUNCHER-WEB-SETTINGS-001（设置）:
     `打开设置 -> 修改游戏/区块链字段 -> 保存/重载 LLM 参数 -> 关闭窗口`
  2. Flow-LAUNCHER-WEB-FEEDBACK-001（反馈成功）:
     `链就绪 -> 打开反馈窗口 -> 输入类型/标题/描述 -> 提交 -> 返回 feedback_id/event_id`
  3. Flow-LAUNCHER-WEB-FEEDBACK-002（反馈失败）:
     `提交反馈 -> 代理层或 runtime 拒绝 -> Web UI 展示结构化错误`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Web 设置中心 | 游戏/区块链配置 + `llm.api_key/base_url/model` | 点击“设置”打开窗口；保存写入浏览器存储；重载读回存储 | `closed -> open -> saved/reloaded` | 配置字段沿用 shared schema 与现有 config 结构 | 仅本地会话可编辑 |
| Web 反馈窗口 | `kind/title/description` | 点击“反馈”打开窗口；点击“提交反馈”调用 `/api/chain/feedback` | `idle -> validating -> submitting -> success/failed` | 标题/描述必填；提交遵循单请求 in-flight 门控 | 链就绪时可提交 |
| 控制面反馈代理 | `/api/chain/feedback` JSON 透传 | 代理到 `/v1/chain/feedback/submit`，返回结构化结果 | `accepted/rejected/proxy_failed` | 不重写 runtime 反馈规则 | 仅受信网络部署可调用 |
- Acceptance Criteria:
  - AC-1: Web 顶部栏 `设置` 与 `反馈` 按钮可点击，不再固定禁用。
  - AC-2: `llm_settings_web.rs` 提供可交互设置窗口，不再立即关闭。
  - AC-3: `feedback_window_web.rs` 提供可交互反馈窗口并执行提交。
  - AC-4: `world_web_launcher` 暴露 `/api/chain/feedback` 并返回结构化结果。
  - AC-5: wasm 反馈提交在成功/失败场景都能显示明确状态文案。
- Non-Goals:
  - 不实现 Web 本地文件落盘回退（保持浏览器约束）。
  - 不在本轮新增反馈附件上传、图片/日志自动打包。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - wasm 启动器通过 `/api/chain/feedback` 向控制面提交反馈。
  - 控制面转发到 runtime `/v1/chain/feedback/submit` 并返回结构化响应。
  - 设置窗口在 Web 端复用 native 布局语义，LLM 配置写入浏览器存储。
- Integration Points:
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/feedback_window_web.rs`
  - `crates/agent_world_client_launcher/src/llm_settings_web.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/feedback_submit_api.rs`
- Edge Cases & Error Handling:
  - 链未就绪：反馈提交前阻断并提示。
  - 请求并发：存在 in-flight 时拒绝重复提交。
  - 上游非 2xx：尽量透传 runtime 返回体；不可解析时返回 `proxy_error`。
  - 浏览器存储不可用：设置保存失败需展示错误提示，不影响主界面继续操作。
- Non-Functional Requirements:
  - NFR-1: Web 反馈提交本地链路 `p95 <= 500ms`；失败路径 `p95 <= 1s` 返回。
  - NFR-2: 设置窗口打开到可编辑状态 `p95 <= 200ms`（本地浏览器）。
  - NFR-3: 新增文件仍满足单文件 < 1200 行约束。
- Security & Privacy:
  - 反馈请求只包含必要文本字段，不附带敏感凭据。
  - LLM 参数仅保存在当前浏览器本地存储，不自动上传。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: Web 设置/反馈可用化 + 控制面反馈代理。
  - v1.1: 增加 Web 反馈提交错误码覆盖测试。
  - v2.0: 视需求评估浏览器端反馈草稿自动恢复。
- Technical Risks:
  - 风险-1: Web 存储权限受浏览器策略影响，导致保存失败。
  - 风险-2: 控制面与 runtime 反馈错误语义不一致，导致 UI 判定歧义。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-021 -> TASK-WORLD_SIMULATOR-048/049 -> `test_tier_required`。
  - 计划验证：
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`
- Decision Log:
  - DEC-LAUNCHER-WEB-PARITY-001: Web 反馈采用“wasm -> world_web_launcher -> world_chain_runtime”代理链路，而非 wasm 直连链状态端口。理由：复用既有控制面与跨端一致性。
  - DEC-LAUNCHER-WEB-PARITY-002: Web 设置中心中的 LLM 参数采用浏览器本地存储，不模拟 native 文件写入。理由：遵循 wasm 运行环境能力边界并保持可用。
