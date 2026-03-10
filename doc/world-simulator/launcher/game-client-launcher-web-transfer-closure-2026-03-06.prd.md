# 客户端启动器 Web 链上转账闭环补齐（2026-03-06）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: 启动器 UI 已完成 native/web 同层复用，但 Web 端转账入口仍为禁用/占位，导致“统一控制面”在资产交互环节断裂。
- Proposed Solution: 在 `world_web_launcher` 增加转账代理 API，并在 launcher wasm 端接入转账表单校验与提交闭环，补齐 Web 转账能力。
- Success Criteria:
  - SC-1: Web 启动器可执行“填写 from/to/amount/nonce -> 提交 -> 返回结果”闭环。
  - SC-2: `world_web_launcher` 提供 `/api/chain/transfer`，并桥接 `world_chain_runtime` 的 `/v1/chain/transfer/submit`。
  - SC-3: 余额不足、nonce 重放、字段非法等错误在 Web UI 中可结构化展示。
  - SC-4: 对应 `test_tier_required` 回归通过，且不引入 native 路径回归。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家（Web）：需要浏览器内直接完成链上转账，不依赖 native 客户端或命令行。
  - 运维/测试人员：需要在 headless 场景通过 Web 控制台复现转账成功与拒绝路径。
- User Scenarios & Frequency:
  - 无 GUI 服务器远程联调（高频）：每日多次提交小额测试转账。
  - 发布前 Web 回归（每次发布）：至少 1 次成功 + 1 次失败路径验证。
- User Stories:
  - PRD-WORLD_SIMULATOR-020: As a Web 启动器玩家, I want to submit blockchain transfers in browser, so that I can complete asset interaction without native tools.
- Critical User Flows:
  1. Flow-LAUNCHER-WEB-TRANSFER-001（成功）:
     `链状态已就绪 -> 打开转账窗口 -> 输入合法 from/to/amount/nonce -> 提交 -> 返回 action_id`
  2. Flow-LAUNCHER-WEB-TRANSFER-002（失败）:
     `提交请求 -> runtime 拒绝（余额不足/nonce 重放/参数非法） -> Web UI 展示 error_code + error`
  3. Flow-LAUNCHER-WEB-TRANSFER-003（门控）:
     `链未就绪 -> 转账按钮禁用或提交被阻断 -> UI 给出可诊断提示`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Web 转账表单 | `from_account_id/to_account_id/amount/nonce` | 点击“提交转账”触发 `/api/chain/transfer` | `idle -> validating -> submitting -> success/failed` | `amount/nonce` 必须为正整数，`from != to` | 仅链就绪时可提交 |
| 控制面转账代理 | `POST /api/chain/transfer` JSON 负载 | 透传到 chain runtime 转账接口，返回结构化结果 | `accepted/rejected/proxy_failed` | 保持 runtime 错误语义，不重写业务规则 | 仅受信网络部署的控制面可调用 |
- Acceptance Criteria:
  - AC-1: wasm UI 显示可操作的转账窗口（字段输入 + 提交按钮 + 成功/失败状态）。
  - AC-2: `world_web_launcher` 新增 `/api/chain/transfer`，可代理提交到 `/v1/chain/transfer/submit`。
  - AC-3: Web 提交失败时可看到 `error_code/error`，成功时可看到 `action_id`。
  - AC-4: 链未就绪或未启用时，转账提交被阻断并显示明确提示。
  - AC-5: `test_tier_required` 覆盖代理 API 与 wasm 提交流程，不破坏现有 start/stop/state 链路。
- Non-Goals:
  - 不实现 Web 端钱包托管、私钥签名、多签、跨链桥接。
  - 不在本轮实现反馈窗口 Web 提交闭环（仅转账专题）。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - wasm 启动器在提交时调用 `world_web_launcher` 的 `/api/chain/transfer`。
  - 控制面服务将请求桥接到 `world_chain_runtime` 的 `/v1/chain/transfer/submit`。
  - 返回结构化响应给 wasm UI，渲染成功/失败状态。
- Integration Points:
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/transfer_window.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
- Edge Cases & Error Handling:
  - 链未启动/不可达：返回结构化失败并保留连接错误上下文。
  - 非法 payload：控制面返回 `invalid_request` 语义，UI 展示字段级错误提示。
  - 上游非 2xx：尽量透传 runtime 返回体；无法解码时返回 `proxy_error`。
  - 并发请求：Web 客户端沿用单请求 in-flight 门控，避免重复提交。
- Non-Functional Requirements:
  - NFR-1: 本地网络下 Web 转账提交 `p95 <= 500ms`（不含链执行确认时间）。
  - NFR-2: `/api/chain/transfer` 在链不可达时 `p95 <= 1s` 返回失败，不得长时间阻塞 UI。
  - NFR-3: Web 与 native 转账错误文案语义保持一致（成功、拒绝、失败三态）。
  - NFR-4: 所有新增 Rust 文件行数保持 < 1200。
- Security & Privacy:
  - 转账请求仅透传必要字段，不新增敏感凭据字段。
  - 错误信息可诊断但不输出密钥或系统隐私数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 补齐 `/api/chain/transfer` + wasm 转账提交与错误展示。
  - v1.1: 追加 Web 转账回归与发布证据沉淀。
  - v2.0: 视需求补齐 Web 反馈提交与更丰富交易可观测。
- Technical Risks:
  - 风险-1: 控制面代理与 runtime 错误码不一致，导致前端判定分叉。
  - 风险-2: wasm 提交流程若未接入 in-flight 门控，可能重复提交。
  - 风险-3: 控制面链状态与 runtime 可达性短时抖动，可能引发误判失败。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-020 -> TASK-WORLD_SIMULATOR-046/047 -> `test_tier_required`。
  - 计划验证：
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher transfer_entry::tests:: -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`
- Decision Log:
  - DEC-LAUNCHER-WEB-TRANSFER-001: 采用“wasm -> world_web_launcher -> world_chain_runtime”的代理链路，而不是 wasm 直连 `chain_status_bind`。理由：Web 端无法可靠复用 native TCP 直连模型，且控制面代理更符合已有架构。
  - DEC-LAUNCHER-WEB-TRANSFER-002: 保持 runtime 作为转账业务规则唯一来源，控制面不复制账本校验逻辑。理由：避免双份规则导致语义漂移。
