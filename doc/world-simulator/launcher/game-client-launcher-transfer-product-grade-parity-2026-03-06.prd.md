# 客户端启动器转账产品级体验与跨端同层前端（2026-03-06）

审计轮次: 5
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.project.md`

## 1. Executive Summary
- Problem Statement: 当前启动器转账能力仍偏工程调试态，native 与 web 行为存在差异（门控策略、交互反馈与提交流程口径不一致），且仅返回 `action_id` 缺少交易历史与最终确认可视化。
- Proposed Solution: 将转账能力升级为产品级体验，统一 native/web 为同一套前端交互实现（同层复用），补齐账户选择、余额辅助、自动 nonce、最终状态可视化与历史记录查询。
- Success Criteria:
  - SC-1: native 与 web 转账窗口复用同一套前端组件与状态机，字段、校验、按钮门控、状态文案一致率 100%。
  - SC-2: 用户可通过账户选择器与余额辅助完成转账，默认无需手工输入 nonce（自动 nonce 覆盖率 >= 95%，手动覆盖保留）。
  - SC-3: 每笔提交后均可看到结构化生命周期状态（`accepted/pending/confirmed/failed/timeout`），不再停留在仅 `action_id` 回执。
  - SC-4: 启动器可查看最近转账历史（按账户过滤、按时间倒序），并支持按 `action_id` 定位。
  - SC-5: `test_tier_required` 回归通过，且不引入 native/web 行为漂移。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：希望在单一入口完成资产查询、转账与结果确认。
  - 运维/测试人员：希望在 headless 场景复现完整转账闭环并快速定位失败原因。
  - 启动器维护者：希望减少 native/web 双实现分叉带来的长期维护成本。
- User Scenarios & Frequency:
  - 日常资产操作：每次启动后 1~5 次转账，需快速完成且可确认结果（高频）。
  - 发布前回归：每个版本至少执行一次“成功 + 拒绝 + 最终状态确认 + 历史检索”闭环（中频）。
  - 故障排查：出现 nonce/余额冲突时需通过历史与状态详情快速诊断（中频）。
- User Stories:
  - PRD-WORLD_SIMULATOR-023: As a 启动器玩家, I want a product-grade transfer experience with account/balance/nonce assistance and final status visibility, so that I can complete transfers reliably on both native and web with the same interaction model.
- Critical User Flows:
  1. Flow-LAUNCHER-TRANSFER-PRO-001（就绪门控）:
     `区块链未就绪 -> 转账入口禁用并展示原因 -> 区块链就绪后入口自动可用`
  2. Flow-LAUNCHER-TRANSFER-PRO-002（自动 nonce 成功路径）:
     `打开转账窗口 -> 选择 from/to 账户 -> 查看余额提示 -> 输入金额 -> 自动填充 nonce -> 提交 -> received accepted`
  3. Flow-LAUNCHER-TRANSFER-PRO-003（最终状态确认）:
     `提交成功返回 action_id -> 进入 pending -> 轮询/订阅状态 -> 进入 confirmed 或 failed -> UI 展示最终结果与原因`
  4. Flow-LAUNCHER-TRANSFER-PRO-004（失败与重试）:
     `提交被拒绝（余额不足/nonce 冲突/参数非法） -> UI 展示结构化错误 -> 一键修正并重试`
  5. Flow-LAUNCHER-TRANSFER-PRO-005（历史检索）:
     `按账户筛选 -> 查看最近转账记录列表 -> 点击记录查看 action_id、金额、状态与时间`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 转账入口门控 | `chain_status`、`chain_enabled`、`config_issues` | 仅当链就绪且配置合法时启用“转账”按钮；否则禁用并提示 | `disabled -> enabled` | 状态以控制面快照为准，默认 1s 刷新 | 只读状态判定，玩家不可绕过 |
| 账户选择与余额辅助 | `from_account_id`、`to_account_id`、`from_balance`、`to_balance(optional)` | 下拉选择账户；选中后展示余额与可转建议 | `idle -> account_selected` | 余额来自链查询；账户列表按最近活跃 + 字典序稳定排序 | 仅展示可见账户，跨账户写操作需显式选择 |
| nonce 策略 | `nonce_mode(auto/manual)`、`nonce`、`next_nonce_hint` | 默认 `auto`，可切换 `manual` 覆盖；提交前校验 | `auto -> manual(optional)` | `auto` 使用服务端建议 nonce；`manual` 必须 > 0 且满足规则 | nonce 校验以 runtime 为准，前端仅做预校验 |
| 转账提交与状态 | `action_id`、`submit_result`、`final_status`、`error_code/error` | 点击提交后进入 in-flight；返回 accepted 后追踪最终状态 | `idle -> validating -> submitting -> accepted -> pending -> confirmed/failed/timeout` | 状态按服务端时间线推进，禁止倒退 | 链未就绪时禁止提交 |
| 交易历史视图 | `history_items[]`（account/from/to/amount/nonce/action_id/status/timestamp） | 支持刷新、按账户筛选、按 action_id 定位 | `empty -> loaded -> filtered` | 默认按 `timestamp desc`；同时刻按 `action_id desc` | 仅查询权限，不含敏感密钥字段 |
| 跨端前端同层复用 | shared transfer panel schema + shared state machine | native/web 均通过同一前端组件渲染与交互 | `single_source` | 由同一实现产出 UI 与状态，不允许平台分叉逻辑 | 平台差异仅限 transport adapter |
- Acceptance Criteria:
  - AC-1: native 与 web 转账 UI 入口门控一致，链未就绪时均不可点击“转账/提交转账”。
  - AC-2: native 与 web 转账表单字段一致，均支持账户选择、余额展示、金额输入、nonce 自动/手动模式。
  - AC-3: 转账提交成功后，UI 必须展示 `action_id` 与 `pending`，并在最终状态到达后更新为 `confirmed/failed/timeout`。
  - AC-4: 转账历史面板可展示最近 N 笔记录（默认 N=50），支持按账户过滤与 `action_id` 检索。
  - AC-5: 所有拒绝路径必须结构化展示 `error_code + error`，且可重试。
  - AC-6: 代码结构上 native/web 转账前端实现统一为单一来源，不再维护两份独立交互逻辑。
  - AC-7: required 回归覆盖成功、拒绝、超时、并发、门控与历史查询路径。
  - AC-8: 本专题可追溯到 `PRD-WORLD_SIMULATOR-023` 与对应任务/测试证据。
- Non-Goals:
  - 不实现钱包托管、私钥管理、多签、跨链桥接。
  - 不在本轮引入手续费市场与交易排序策略（mempool 策略）。
  - 不在本轮改造 runtime 共识语义，仅扩展查询与可观测能力。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - 前端层：`agent_world_client_launcher` 提供单一 transfer panel 组件，native/wasm 通过平台适配层复用。
  - 控制面层：`world_web_launcher` 统一提供转账提交、状态查询、历史查询、余额查询 API。
  - 运行时层：`world_chain_runtime` 继续作为转账业务规则唯一来源，负责 nonce/余额/账户合法性校验与状态推进。
- Integration Points:
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/transfer_window.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_web_launcher/transfer_query_proxy.rs`
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api_tests.rs`
  - `crates/agent_world/src/runtime/world/resources.rs`
- Edge Cases & Error Handling:
  - 链未就绪：入口禁用 + 诊断提示；禁止“点了再报错”的晚失败体验。
  - 余额不足：提交拒绝后保留当前输入并高亮余额差额提示。
  - nonce 冲突：自动模式下触发刷新 nonce hint；手动模式保持用户输入并标注冲突。
  - 接口超时：提交或状态查询超时进入 `timeout`，支持手动重查与重试。
  - 状态悬挂：`accepted` 长时间未进入最终态时，显示“待确认”并提供 `action_id` 复制。
  - 并发提交：单账户同窗口并发提交受 in-flight 门控，避免重复发送。
  - 空历史：显示空态文案与引导，不阻断新建转账。
  - 非法响应：解析失败时进入可重试错误态并写诊断日志。
- Non-Functional Requirements:
  - NFR-1: native/web 转账前端逻辑复用率 100%（单一实现源），跨端转账行为一致性差异为 0。
  - NFR-2: 本地链路下转账提交 API `p95 <= 500ms`；失败路径 `p95 <= 1s`。
  - NFR-3: 历史列表查询 `p95 <= 300ms`（最近 50 条，本地链路）。
  - NFR-4: 提交后状态刷新间隔默认 <= 1s，最终态可见延迟 <= 2 个轮询周期（本地链路）。
  - NFR-5: 新增或改造 Rust 文件保持单文件 < 1200 行。
- Security & Privacy:
  - 查询与提交接口仅包含最小必要字段，不传输密钥类敏感信息。
  - 历史与状态响应仅暴露可公开账本字段，避免额外隐私泄露。
  - 错误信息可诊断但不暴露内部敏感路径或凭据内容。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成产品级转账 PRD 与任务拆解，冻结跨端统一交互规范。
  - v1.1: 落地共享前端 transfer panel + 就绪门控 + 自动 nonce + 余额辅助。
  - v2.0: 落地历史/最终状态查询与可观测指标增强。
- Technical Risks:
  - 风险-1: 当前 launcher Web 端保留全局单请求 in-flight 门控，状态轮询与其他请求串行，可能在高频刷新场景下放大等待抖动。
  - 风险-2: 运行时转账状态/历史跟踪为进程内存态，不跨重启持久化；重启后历史可见性受限。
  - 风险-3: 自动 nonce 策略与多端并发写入场景存在竞争窗口，需要明确定义冲突回退策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-023 -> TASK-WORLD_SIMULATOR-052/053 -> T1/T2/T3 -> `test_tier_required` + `test_tier_full`。
  - 已执行验证（2026-03-07）：
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --tests --features test_tier_required transfer_submit_api::tests:: -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --tests --features test_tier_full transfer_submit_api::tests:: -- --nocapture`
- Decision Log:
  - DEC-LAUNCHER-TRANSFER-PRO-001: 转账前端强制单一实现（同层复用），native/web 仅保留 transport 适配差异。理由：从根源消除跨端行为漂移。
  - DEC-LAUNCHER-TRANSFER-PRO-002: nonce 默认自动分配，保留手动覆盖作为高级路径。理由：降低普通用户失败率，同时保留调试能力。
  - DEC-LAUNCHER-TRANSFER-PRO-003: 将“提交 ACK（action_id）”与“最终状态确认”拆分展示。理由：明确异步链路阶段，减少“已提交=已成功”误解。
  - DEC-LAUNCHER-TRANSFER-PRO-004: 将链就绪门控前置到按钮可用态，不再依赖提交后失败提示。理由：减少无效操作与错误噪声。
