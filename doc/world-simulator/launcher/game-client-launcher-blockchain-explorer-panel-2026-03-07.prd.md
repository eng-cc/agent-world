# 客户端启动器区块链浏览器面板（2026-03-07）

审计轮次: 1
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-panel-2026-03-07.prd.project.md`

## 1. Executive Summary
- Problem Statement: 启动器当前虽有转账入口，但缺少“链浏览器”可视化入口，玩家无法在同一界面查看链高度、区块哈希与交易状态明细，排障与验证依赖命令行。
- Proposed Solution: 先补齐链浏览器最小 RPC（overview/transactions/transaction），再在 native/web 共用启动器中新增“区块链浏览器”面板，提供概览、过滤检索与交易详情。
- Success Criteria:
  - SC-1: `world_chain_runtime` 提供可消费的浏览器 RPC，覆盖总览、交易列表、交易详情三类查询。
  - SC-2: `world_web_launcher` 暴露 `/api/chain/explorer/*` 代理，native/web 客户端统一消费同一控制面。
  - SC-3: `agent_world_client_launcher` 新增“区块链浏览器”面板，支持按账户/状态过滤与按 `action_id` 明细查询。
  - SC-4: 浏览器面板在链未就绪时给出结构化错误提示，不出现 panic 或卡死。
  - SC-5: `test_tier_required` 回归通过，且不回归既有转账/反馈/状态链路。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要快速确认链是否出块、交易是否最终确认。
  - 测试/运维人员：需要无需 CLI 的浏览器视图定位失败交易与状态漂移。
  - 启动器维护者：需要复用现有控制面并保持 native/web 体验一致。
- User Scenarios & Frequency:
  - 日常运行中查看链状态：每次启动后 1~3 次（高频）。
  - 转账提交后核对最终状态：每笔转账后 1 次（高频）。
  - 发布回归核验：每个版本至少 1 次（中频）。
- User Stories:
  - PRD-WORLD_SIMULATOR-024: As a 启动器玩家, I want a blockchain explorer panel in launcher, so that I can inspect chain overview and transaction details without command-line tools.
- Critical User Flows:
  1. Flow-LAUNCHER-EXPLORER-001（总览查询）:
     `打开区块链浏览器 -> 刷新 overview -> 查看 latest/committed/network height 与 last block hash`
  2. Flow-LAUNCHER-EXPLORER-002（交易过滤）:
     `输入 account_id + 选择 status -> 查询交易列表 -> 按时间倒序浏览`
  3. Flow-LAUNCHER-EXPLORER-003（明细定位）:
     `输入 action_id 或点击列表项 -> 查询 transaction detail -> 查看最终状态与错误码`
  4. Flow-LAUNCHER-EXPLORER-004（未就绪失败路径）:
     `链未就绪 -> 查询返回结构化错误 -> 面板展示错误并允许重试`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Explorer Overview RPC | `latest_height`、`committed_height`、`network_committed_height`、`last_block_hash`、`transfer_* counters` | 点击“刷新”发起 `/api/chain/explorer/overview` | `idle -> loading -> ready/failed` | counters 由运行时追踪记录实时聚合 | 查询只读 |
| Explorer Transactions RPC | `account_filter`、`status_filter`、`limit`、`items[]` | 点击“应用过滤”发起 `/api/chain/explorer/transactions` | `idle -> loading -> ready/failed` | 列表按 `submitted_at desc` + `action_id desc` | 查询只读 |
| Explorer Transaction RPC | `action_id`、`status`、`error_code/error` | 点击“查询详情”发起 `/api/chain/explorer/transaction` | `idle -> loading -> ready/failed` | 单条明细按 `action_id` 精确查询 | 查询只读 |
| 启动器浏览器面板 | `overview`、`filters`、`transactions`、`selected_transaction` | 打开窗口、刷新、过滤、详情查询 | `closed/open` + 子状态机 | 与转账面板共享 `WebTransferLifecycleStatus` 语义 | 仅链就绪可操作 |
- Acceptance Criteria:
  - AC-1: `world_chain_runtime` 支持 `GET /v1/chain/explorer/overview`、`GET /v1/chain/explorer/transactions`、`GET /v1/chain/explorer/transaction`。
  - AC-2: `world_web_launcher` 提供对应 `/api/chain/explorer/*` 代理接口并保持结构化错误语义。
  - AC-3: 启动器新增“区块链浏览器”入口与面板，native/web 同源 UI 行为一致。
  - AC-4: 列表支持账户过滤、状态过滤（accepted/pending/confirmed/failed/timeout）与默认 `limit=50`。
  - AC-5: 详情查询支持 `action_id` 输入与列表项点击两种路径。
  - AC-6: 失败路径展示 `error_code + error`，且不阻断后续重试。
  - AC-7: 对应任务与测试可追溯到 `PRD-WORLD_SIMULATOR-024`。
- Non-Goals:
  - 不在本轮实现完整区块分页浏览或 Merkle 证明校验可视化。
  - 不引入钱包管理、多签、跨链桥等能力。
  - 不改造共识语义，仅做查询与展示能力补齐。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - 运行时层: `world_chain_runtime` 提供 explorer 查询 RPC（链总览 + 交易列表 + 交易详情）。
  - 控制面层: `world_web_launcher` 将 explorer 查询代理为 `/api/chain/explorer/*`。
  - 客户端层: `agent_world_client_launcher` 新增 explorer window，消费控制面接口并展示结构化结果。
- Integration Points:
  - `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api_tests.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/explorer_window.rs`
- Edge Cases & Error Handling:
  - `status` 过滤参数非法时返回 `invalid_request`，并保持 400 语义。
  - `action_id` 非正整数时拒绝查询并返回结构化错误。
  - 链禁用或不可达时返回 `chain_disabled/proxy_error`，前端显示错误并保留重试。
  - 空交易集时返回空列表与 `total=0`，面板展示空态文案。
  - 运行时锁失败或响应解析失败时返回 `internal_error/proxy_error`，日志可诊断。
- Non-Functional Requirements:
  - NFR-1: explorer 查询接口本地链路 `p95 <= 500ms`（limit=50）。
  - NFR-2: 浏览器面板默认刷新周期 1s，状态可见延迟 <= 2 个轮询周期。
  - NFR-3: 新增 Rust 文件保持单文件 < 1200 行。
  - NFR-4: native/web 浏览器面板字段、按钮、错误语义一致率 100%。
- Security & Privacy:
  - explorer 仅暴露账本可公开字段，不输出私钥或敏感配置。
  - 错误信息保留可诊断性但不泄露本地敏感路径与凭据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成专题 PRD 建模与任务拆解。
  - v1.1: 落地 runtime explorer RPC + control plane 代理。
  - v2.0: 落地 launcher explorer 面板与跨端回归。
- Technical Risks:
  - 风险-1: 当前交易追踪基于进程内存，重启后历史可见性受限。
  - 风险-2: 全局单请求 in-flight 门控可能让 explorer 刷新与其他操作串行等待。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-024 -> TASK-WORLD_SIMULATOR-054/055/056 -> `test_tier_required`。
  - 计划验证命令:
    - `./scripts/doc-governance-check.sh`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_chain_runtime transfer_submit_api::tests:: -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown`
- Decision Log:
  - DEC-LAUNCHER-EXPLORER-001: 先补 RPC，再接 UI。理由：避免 UI 与接口语义反复返工。
  - DEC-LAUNCHER-EXPLORER-002: explorer 复用 transfer tracker 与 lifecycle 状态枚举。理由：降低重复实现与语义分叉风险。
  - DEC-LAUNCHER-EXPLORER-003: native/web 继续走统一控制面 API。理由：保持跨端行为一致并降低维护成本。
