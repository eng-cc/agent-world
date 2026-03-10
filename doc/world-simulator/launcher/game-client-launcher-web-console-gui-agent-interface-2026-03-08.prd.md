# 客户端启动器 Web Console GUI Agent 全量接口（2026-03-08）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 当前 `world_web_launcher` 已有多组分散 API（启停、反馈、转账、浏览器查询），但缺少面向 GUI Agent 的“单一机器接口”与能力声明，导致自动化代理需要手工拼接多路端点与状态语义。
- Proposed Solution: 在 Web Console 增加 `gui-agent` 命名空间，提供“能力发现 + 统一动作执行 + 状态别名”接口，将人工可做的启动器功能映射为可编排动作。
- Success Criteria:
  - SC-1: GUI Agent 可通过统一入口覆盖人工操作的全功能集合（游戏/链启停、反馈、转账、浏览器查询、状态读取）。
  - SC-2: 每个动作返回结构化 `ok/error_code/error + state + data`，无需依赖 UI 文案解析。
  - SC-3: API 对错误类型稳定分层（`invalid_request/chain_disabled/proxy_error/...`），失败可自动重试或转人工。
  - SC-4: 新接口不破坏既有 `/api/*` 路由，native/web 客户端行为无回归。

## 2. User Experience & Functionality
- User Personas:
  - GUI Agent 编排器：以机器方式调用启动器全部能力，替代人工点击。
  - 运维/测试人员：在 headless 环境通过脚本与 Agent 完成一键化回归。
  - 启动器维护者：维护统一可演进的控制面契约，降低接口漂移风险。
- User Scenarios & Frequency:
  - 自动化巡检与回归：每次版本候选至少 1 次全链路执行。
  - 日常远程运维：按需触发（通常每天多次状态查询 + 少量写操作）。
- User Stories:
  - PRD-WORLD_SIMULATOR-031: As a GUI Agent 编排器, I want one machine-oriented API surface in web console, so that I can execute every manual launcher operation without UI-dependent parsing.
- Critical User Flows:
  1. Flow-GA-001（能力发现）:
     `GET /api/gui-agent/capabilities -> 读取动作清单/参数约束 -> Agent 生成执行计划`
  2. Flow-GA-002（统一动作执行）:
     `POST /api/gui-agent/action(action=start_game/start_chain/...) -> 返回结构化结果 + 最新 state`
  3. Flow-GA-003（链上功能闭环）:
     `action=submit_transfer/submit_feedback + explorer 查询动作 -> 输出结构化成功或错误签名`
  4. Flow-GA-004（失败恢复）:
     `收到 invalid_request/proxy_error/chain_disabled -> Agent 根据 error_code 分支重试/降级/告警`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 能力发现 | `api_version/actions[]/query_targets[]` | `GET /api/gui-agent/capabilities` 返回完整动作与查询目标 | `ready`（只读） | actions 固定枚举，避免动态歧义 | 受信网络内可读 |
| 状态别名 | `StateSnapshot` | `GET /api/gui-agent/state` 等价读取当前控制面状态 | `polling` 常驻 | 与 `/api/state` 同步字段语义 | 受信网络内可读 |
| 统一动作执行 | `action + payload` | `POST /api/gui-agent/action` 执行启停/反馈/转账/查询动作 | `idle -> running/stopped/...`（取决于动作） | 每次动作后返回最新 `state` 快照 | 受信网络内可写 |
| 查询动作代理 | `query_target + query` | 通过固定目标映射到 explorer/transfer 查询 | `idle -> loading -> ready/failed` | 目标白名单映射，拒绝任意路径穿透 | 查询只读 |
| 错误契约 | `ok/error_code/error/data/state` | 任意动作均返回统一结构，便于 Agent 判定 | `success/failed` | `error_code` 稳定可枚举 | 机器调用优先 |
- Acceptance Criteria:
  - AC-1: 新增 `GET /api/gui-agent/capabilities`，返回 GUI Agent 可调用动作全集。
  - AC-2: 新增 `GET /api/gui-agent/state`，语义与 `/api/state` 对齐。
  - AC-3: 新增 `POST /api/gui-agent/action`，支持以下动作：
    - 控制类：`start_game`、`stop_game`、`start_chain`、`stop_chain`
    - 提交类：`submit_transfer`、`submit_feedback`
    - 查询类：`query_transfer_accounts`、`query_transfer_status`、`query_transfer_history`
    - 浏览器查询类：`query_explorer_overview`、`query_explorer_transactions`、`query_explorer_transaction`、`query_explorer_blocks`、`query_explorer_block`、`query_explorer_txs`、`query_explorer_tx`、`query_explorer_search`、`query_explorer_address`、`query_explorer_contracts`、`query_explorer_contract`、`query_explorer_assets`、`query_explorer_mempool`
  - AC-4: 所有动作统一返回 `{ ok, action, error_code?, error?, data?, state }`。
  - AC-5: `query_target` 采用白名单映射到 runtime 目标路径，不允许任意 URL 透传。
  - AC-6: 既有 `/api/state`、`/api/start`、`/api/stop`、`/api/chain/*` 与静态资源服务行为不回归。
- Non-Goals:
  - 不在本轮引入鉴权/RBAC（继续沿用受信网络部署前提）。
  - 不重写现有 web 客户端 UI 交互层。
  - 不改造 `world_viewer_live` WebSocket 协议。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: GUI Agent 仅需 HTTP JSON 能力；不要求浏览器 DOM 自动化。
- Evaluation Strategy: 以动作成功率、错误可分类率、全功能覆盖率评估（对照人工操作 checklist）。

## 4. Technical Specifications
- Architecture Overview:
  - `world_web_launcher` 新增 `gui_agent_api` 子模块，负责动作协议解析与统一响应封装。
  - 路由层增加 `/api/gui-agent/*` 命名空间，复用现有 `control_plane` 与 `transfer_query_proxy` 能力。
- Integration Points:
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_web_launcher/gui_agent_api.rs`（新增）
  - `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`
- Edge Cases & Error Handling:
  - 未知 action：返回 `invalid_request`。
  - payload 缺失或 JSON 不合法：返回 `invalid_request`。
  - 链功能禁用：返回 `chain_disabled`，并带最新 state。
  - 查询代理失败：返回 `proxy_error`，并保留上游错误文本。
  - 启停冲突（重复启动/停止）：返回结构化失败并保持状态可诊断。
- Non-Functional Requirements:
  - NFR-1: `POST /api/gui-agent/action`（本地链路）`p95 <= 500ms`（查询类）/ `p95 <= 1s`（提交与控制类）。
  - NFR-2: 动作返回结构体字段稳定；新增字段仅追加，不破坏既有键名语义。
  - NFR-3: 单次动作后必须回传最新 `state`，避免 Agent 二次探测开销。
- Security & Privacy:
  - 保持“受信网络部署”原则；接口返回不得包含私钥或敏感凭据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 文档建模 + 动作协议冻结。
  - v1.1: `gui-agent` 路由与动作执行模块落地。
  - v1.2: 回归测试、模块 PRD 追溯与 devlog 收口。
- Technical Risks:
  - 风险-1: 动作枚举增长过快导致路由维护复杂度上升。
  - 风险-2: 统一返回结构若与历史错误语义不一致，可能引发 Agent 误判。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_SIMULATOR-031 | TASK-WORLD_SIMULATOR-091/092 | `test_tier_required` | `./scripts/doc-governance-check.sh` + `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_web_launcher` | web console 机器控制面、人工操作可替代性、既有控制面兼容性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-GA-001 | 在现有控制面上新增 `/api/gui-agent/*` 命名空间并复用内部能力 | 直接要求 GUI Agent 调用分散 `/api/*` 旧路由 | 单入口与统一返回结构更利于 Agent 稳定编排与错误分流。 |
| DEC-GA-002 | 使用白名单 `query_target` 映射查询 | 暴露任意 runtime URL 透传接口 | 白名单可避免路径注入和语义漂移。 |
| DEC-GA-003 | 动作执行后总是返回 `state` 快照 | 仅返回动作结果，状态另查 | 减少 Agent 往返调用次数并提升一致性。 |
