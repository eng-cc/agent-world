# OpenClaw 本地 HTTP Provider 接入 world-simulator 首期方案（2026-03-12）

- 对应设计文档: `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.design.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: `Decision Provider` 标准层已经明确了“外部 provider 可参与 Agent 决策，但不得替代 runtime 权威”的边界；但若要回答“安装在用户机器上的 `OpenClaw` 怎么玩这个游戏”，还缺一份面向真实用户安装场景的接入方案，尤其是本地发现、握手、配置、玩家-agent 绑定、决策接口、失败恢复与最小可玩范围。
- Proposed Solution: 首期采用“`OpenClaw` 本地进程 + `localhost HTTP/JSON`”方案。`OpenClaw` 在用户机器上以本地服务形式运行，仅监听 `127.0.0.1`；world-simulator 侧通过 `OpenClawAdapter` 调用其本地 HTTP API，发送结构化 `DecisionRequest`，接收结构化 `DecisionResponse`。运行时仍由本地 runtime/kernel 权威执行动作、校验规则并产出 trace。Launcher / Viewer 仅负责配置、发现与可观测性展示。
- Success Criteria:
  - SC-1: 用户在本机安装并启动 `OpenClaw` 后，可在 launcher 中发现并选择 `OpenClaw(Local HTTP)` 作为 agent provider。
  - SC-2: 首期 `test_tier_required` 依赖 `localhost HTTP/JSON` 完成单一低频 NPC 的 `wait` / `wait_ticks` / `move_agent` / `speak_to_nearby` / `inspect_target` / `simple_interact` 决策闭环；其中后三者先以 lightweight event 语义落地，并继续受 parity 门禁约束。
  - SC-3: 若本机未安装、未启动、版本不兼容或握手失败，launcher 必须提供明确诊断与回退路径，不得阻断内置 provider 使用。
  - SC-4: 所有 `OpenClaw` 输出必须经 action schema 白名单和 runtime 校验后才能执行；非法输出一律映射为 `Wait` 或 `ActionRejected`。
  - SC-5: `OpenClaw` 决策过程可映射到 `AgentDecisionTrace`，在 viewer / QA 调试面中可见 provider 名称、延迟、错误与最近一次结构化决策。
  - SC-6: 首期 required 验证不依赖真实 `OpenClaw` 网络环境，必须可由 mock local HTTP server 覆盖。

## 2. User Experience & Functionality
- User Personas:
  - 玩家 / 制作人：希望在自己电脑上装好 `OpenClaw` 后，能让游戏里的部分 agent 由它驱动，并知道当前是否正常连接。
  - `agent_engineer`：需要稳定的本地传输协议与 adapter，避免把 provider 细节泄露进模拟内核。
  - `viewer_engineer`：需要在 launcher / viewer 中展示 provider 发现、连接、版本、延迟与错误状态。
  - `qa_engineer`：需要使用 mock 本地 HTTP 服务验证协议与失败签名。
- User Scenarios & Frequency:
  - 首次配置：用户首次安装 `OpenClaw` 后，在 launcher 中选择本地 provider 并完成一次 agent 绑定。
  - 日常试玩：用户启动 launcher / game 后，默认自动探测本机 `OpenClaw` 是否在线。
  - 故障恢复：本机服务未启动、token 不匹配、版本过旧时，用户根据 launcher 提示修复后重试。
- User Stories:
  - PRD-WORLD_SIMULATOR-037: As a 玩家 / 制作人, I want an `OpenClaw(Local HTTP)` provider mode that runs on my machine and can control low-frequency game agents through localhost, so that I can try external agent-driven gameplay without deploying remote services or weakening runtime authority.
- Critical User Flows:
  1. Flow-OC-LOCAL-001（首次安装与发现）:
     `用户安装并启动 OpenClaw 本地服务 -> launcher 探测 localhost provider -> 显示版本/状态 -> 用户选择 OpenClaw(Local HTTP)`。
  2. Flow-OC-LOCAL-002（玩家绑定与启动）:
     `选择 provider -> 绑定 player_id / agent_id 或 NPC profile -> 启动游戏 -> runtime 为目标 agent 使用 OpenClawAdapter`。
  3. Flow-OC-LOCAL-003（决策闭环）:
     `ObservationEnvelope -> POST /v1/world-simulator/decision -> DecisionResponse -> runtime validate/execute -> feedback/trace`。
  4. Flow-OC-LOCAL-004（失败恢复）:
     `provider offline / version mismatch / timeout / invalid action -> launcher/viewer 告警 -> fallback 内置 provider 或禁用 OpenClaw provider`。
  5. Flow-OC-LOCAL-005（用户可观测）:
     `viewer 右侧调试面显示 provider=OpenClaw(Local HTTP)、连接状态、最近延迟、最后错误、最近动作摘要`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Provider 发现 | `provider_id/version/capabilities/health` | launcher 自动探测或手动刷新本机 provider | `offline -> discovered -> ready` | 仅探测 allowlist 端口/路径 | 仅本机回环地址 |
| Provider 选择 | `provider_mode=openclaw_local_http` | 用户在设置中心选择本地 OpenClaw | `builtin -> openclaw_local_http` | provider 配置按 profile 存储 | 仅本地用户可配 |
| 决策请求 | `DecisionRequest` | runtime 对目标 agent 发起一次决策请求 | `observed -> requesting -> responded` | 每 tick 每 agent 至多一请求 | 仅本地 runtime 发起 |
| 结构化决策 | `decision/action_ref/args/diagnostics` | provider 返回 wait/act | `responded -> validated -> executed/rejected` | 动作必须先过 schema | runtime 权威裁定 |
| 状态反馈 | `FeedbackEnvelope` | runtime 把执行结果回写 provider | `executed/rejected -> feedback_sent` | 顺序跟随 action_id | 仅对应会话可写 |
| 故障回退 | `error_code/error/detail/retryable` | launcher/viewer 显示错误并允许回退 | `ready -> degraded -> fallback` | retryable 错误优先重试一次 | 不得自动切远端 |
- Acceptance Criteria:
  - AC-1: 文档定义 `OpenClaw(Local HTTP)` 的用户安装与接入路径，覆盖发现、选择、绑定、启动、调试与恢复。
  - AC-2: 文档冻结最小本地 HTTP 协议集合：`/v1/provider/info`、`/v1/provider/health`、`/v1/world-simulator/decision`、`/v1/world-simulator/feedback`。
  - AC-3: 文档明确首期仅监听 `127.0.0.1`，不开放局域网与公网访问。
  - AC-4: 文档明确首期可玩动作白名单与非目标范围，并要求非法输出统一降级处理。
  - AC-5: 文档明确 launcher / viewer 所需的状态字段与用户提示文案边界。
  - AC-6: 文档定义 required/full 验证矩阵，要求可用 mock local HTTP server 覆盖首期协议。
- Non-Goals:
  - 不在首期引入远程 `OpenClaw` provider、云托管 provider 或公网隧道。
  - 不在首期让 `OpenClaw` 直接执行高频战斗、经济关键路径或批量 agent 群控。
  - 不在首期引入反向 tool callback、双向流式 event feed 或复杂 OAuth 登录。
  - 不在首期把 `OpenClaw` 变成 launcher / viewer 的统一控制面；它只负责世界内 agent 决策。

## 3. AI System Requirements (If Applicable)
- Tool Requirements:
  - `OpenClaw` 首期只需提供本地 HTTP JSON 接口，不要求浏览器 DOM 自动化。
  - provider 输出必须是结构化动作，而不是自由文本。
  - provider 最好导出本轮消息摘要、tool 摘要、延迟与错误信息，以便映射到 trace。
- Evaluation Strategy:
  - `test_tier_required`: 文档建模 + mock local HTTP provider + adapter contract tests + error policy tests。
  - `test_tier_full`: 真实 `OpenClaw(Local HTTP)` 单 NPC 闭环试点，验证动作有效率、延迟、trace 完整度与用户可恢复性。
- Local Runtime Requirements:
  - launcher 与 game runtime 必须支持独立启动；本地 `OpenClaw` 不要求由游戏进程托管。
  - 若用户配置“启动游戏时自动探测本地 OpenClaw”，探测失败不得阻断游戏本身启动。

## 4. Technical Specifications
- Architecture Overview:
  - `OpenClaw`：用户机上独立运行的本地进程，只监听 `127.0.0.1:<port>`。
  - `OpenClawAdapter`：world-simulator 内的 provider adapter，实现 `DecisionProvider`/`AgentBehavior facade`。
  - `Launcher`：负责发现 provider、保存本地配置、展示状态与错误、允许用户启用/禁用本地 provider。
  - `Viewer`：负责展示 trace、provider 状态、最近错误与最近动作摘要。
  - `Runtime/Kernel`：继续负责动作校验、执行、事件与状态演化。
- Integration Points:
  - `crates/agent_world/src/simulator/agent.rs`
  - `crates/agent_world/src/simulator/memory.rs`
  - `crates/agent_world_proto/src/viewer.rs`
  - `crates/agent_world_client_launcher/src/*`
  - `crates/agent_world/src/bin/world_web_launcher/gui_agent_api.rs`
- Local HTTP Endpoints:
  - `GET /v1/provider/info`
    - 返回 `provider_id/name/version/protocol_version/capabilities/supported_action_sets`。
  - `GET /v1/provider/health`
    - 返回 `ok/status/uptime_ms/last_error/queue_depth`。
  - `POST /v1/world-simulator/decision`
    - 请求体：`DecisionRequest`。
    - 响应体：`DecisionResponse`。
  - `POST /v1/world-simulator/feedback`
    - 请求体：`FeedbackEnvelope`。
    - 响应体：`ok/error_code/error`。
- Discovery & Configuration:
  - 默认探测地址：`127.0.0.1:5841`（可配置）。
  - launcher 设置项：
    - `provider_mode`：`builtin_llm` / `openclaw_local_http`
    - `openclaw_base_url`
    - `openclaw_auth_token`（可选；若配置则仅本地保存）
    - `openclaw_auto_discover`
    - `openclaw_connect_timeout_ms`
    - `openclaw_agent_profile`
  - profile 约定：首期 `P0` / parity / experimental 试点默认使用 `agent_world_p0_low_freq_npc`；若 provider 不识别该 profile，必须返回结构化 `unsupported_agent_profile`，禁止静默改用通用玩法。
  - 发现逻辑：优先读取显式配置；若未配置且开启 auto-discover，则探测默认地址。
- DecisionRequest Shape:
  - 顶层字段：`request_id/agent_id/world_time/provider_session_id?/provider_config_ref?/agent_profile?/timeout_ms`
  - `observation`: 当前可见世界状态摘要、附近实体、最近事件、目标与资源摘要。
  - `memory`: 短期记忆摘要、长期记忆命中结果、最近失败动作。
  - `action_catalog`: 动作白名单、参数 schema、枚举值范围、cooldown / cost hint。
  - `player_context`: `player_id`、是否允许外部 provider 接管、绑定关系版本。
  - `trace_context`: 是否要求 provider 返回 transcript/tool summary/diagnostics。
  - `agent_profile`: provider-side 玩法 profile / skill 标识；首期 required 路径至少支持 `agent_world_p0_low_freq_npc`。
- DecisionResponse Shape:
  - `ok`
  - `decision`: `wait` / `wait_ticks` / `act`
  - `action_ref`：仅当 `decision=act` 时出现
  - `args`
  - `diagnostics`: `provider/model/latency_ms/retry_count`
  - `trace_payload`: `messages/tool_calls/tool_results/summary/error`
  - `error_code/error/retryable`
- Phase-1 Action Whitelist:
  - `wait`
  - `wait_ticks`
  - `move_agent`
  - `speak_to_nearby`（lightweight speech event）
  - `inspect_target`（lightweight inspection event）
  - `simple_interact`（lightweight interaction event）
- Error Handling & Fallback:
  - `connection_refused` / `provider_unreachable`: launcher 显示“本地 OpenClaw 未启动”，允许一键切回内置 provider。
  - `version_mismatch`: 阻止启用该 provider，并显示期望协议版本。
  - `timeout`: 本轮决策降级为 `Wait`，若连续超时达到阈值则 provider 状态变 `degraded`。
  - `invalid_action_schema`: 直接 `ActionRejected` 并记录到 trace。
  - `unsupported_semantic_action`: 对于不在 phase-1 白名单内、或 target_kind / payload 不满足当前 lightweight 语义约束的 intent，required 路径必须降级为 `Wait` 并记录结构化错误，禁止伪装为已执行成功。
  - `unsupported_agent_profile`: provider 标记为 `misconfigured`，launcher / parity bench 必须提示用户切回 builtin 或修正 profile。
  - `auth_failed`: provider 标记为 `unauthorized`，要求用户更新本地 token。
- Non-Functional Requirements:
  - NFR-1: 本地 HTTP 仅绑定 `127.0.0.1`，默认不使用 `0.0.0.0`。
  - NFR-2: `GET /info` 与 `GET /health` 本地探测 `p95 <= 200ms`。
  - NFR-3: 首期单次 `decision` 本地请求 `p95 <= 3s`。
  - NFR-4: 首期 provider 错误不得使 runtime tick 卡死；超时后必须回落为可继续推进的状态。
  - NFR-5: mock local HTTP provider 必须可用于 CI / required regression。
  - NFR-6: `DecisionRequest.agent_profile` 必须可经 `ProviderBackedAgentBehavior -> OpenClawAdapter -> local HTTP` 完整透传，并体现在 parity summary / trace 归档中。
- Security & Privacy:
  - 仅接受 loopback 地址；launcher 对 base URL 做 host allowlist 校验。
  - 不向 provider 暴露私钥、完整 auth proof 或内部存储路径。
  - `openclaw_auth_token` 如启用，只存本地配置，不回显在 viewer / logs。
  - 用户必须显式开启 `OpenClaw(Local HTTP)` 模式，默认仍使用内置 provider。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1 (2026-03-12): 完成本地 HTTP 接入方案建模。
  - M2: 落地 provider config、discovery/health-check 与 mock local HTTP contract tests。
  - M3: 实现 `OpenClawAdapter` request/response/feedback 映射。
  - M4: 在 launcher / viewer 加入 provider 状态、错误与 trace 摘要面板。
  - M5: 单低频 NPC 实机试点，并决定是否扩展动作集。
- Technical Risks:
  - 风险-1: 本地 `OpenClaw` 的实际接口与假定协议不完全一致，需要 adapter 额外归一化。
  - 风险-2: 用户机上本地端口冲突、杀毒软件、权限限制可能导致 provider 探测失败。
  - 风险-3: 若 observation 过大，请求体会膨胀；需要摘要策略和字段预算。
  - 风险-4: 若首期就支持反向 callback，会让本地安全边界复杂化；因此本方案明确首期只做单次 request/response。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_SIMULATOR-037 | TASK-WORLD_SIMULATOR-113 | `test_tier_required` | `./scripts/doc-governance-check.sh` | 模块文档入口、专题索引、本地 HTTP 方案建模 |
| PRD-WORLD_SIMULATOR-037 | T1/T2/T3 | `test_tier_required` | mock local HTTP server + adapter contract tests + launcher config tests | provider 发现、握手、决策 contract、失败回退 |
| PRD-WORLD_SIMULATOR-037 | T4/T5 | `test_tier_full` | 真实 `OpenClaw(Local HTTP)` 单 NPC 闭环试点 | 玩家安装路径、体验可行性、trace 完整度 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-OC-LOCAL-001 | 首期采用 `localhost HTTP/JSON` 作为本地外脑传输层 | 首期直接使用 named pipe / UDS / stdio | HTTP 更易调试、跨平台、便于 launcher/viewer 共享健康检查与状态展示。 |
| DEC-OC-LOCAL-002 | `OpenClaw` 作为独立本地进程运行 | 由游戏进程托管/内嵌 OpenClaw 生命周期 | 独立进程更符合用户安装与升级路径，也避免把外部 provider 生命周期绑死到游戏进程。 |
| DEC-OC-LOCAL-003 | 首期只做 `request -> response -> feedback` 单次本地调用 | 首期就做反向 callback、流式工具调用、双向会话总线 | 先用最小协议跑通用户可玩闭环，降低首期安全与工程复杂度。 |
| DEC-OC-LOCAL-004 | 默认仅开放低频动作白名单 | 一开始就开放所有世界动作 | 先收敛风险、验证协议和用户路径，再决定是否扩面。 |
