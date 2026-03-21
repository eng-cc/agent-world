# 客户端启动器区块链浏览器公共主链视角 P1（地址/合约/资产/内存池，2026-03-08）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.project.md`

审计轮次: 6

## 1. Executive Summary
- Problem Statement: 当前启动器浏览器已具备 `blocks/txs/search`，但缺少地址页、合约页、Token/NFT 资产页与 mempool 视图，仍不足以支持“公共主链状态全景查询”。
- Proposed Solution: 在 runtime explorer 索引基础上扩展 P1 查询接口（address/contracts/contract/assets/mempool），由 `oasis7_web_launcher` 代理透传，并在 native/web 共用启动器浏览器面板新增四类视图。
- Success Criteria:
  - SC-1: runtime 提供 `GET /v1/chain/explorer/address`、`/contracts`、`/contract`、`/assets`、`/mempool`。
  - SC-2: 地址页支持余额、nonce、关联交易分页，并可查询 pending 状态。
  - SC-3: 合约页可查看系统合约目录与单合约详情（当前链能力边界显式暴露）。
  - SC-4: 资产页可查看主 token 供应与账户持仓，NFT 能力状态可见（是否支持）。
  - SC-5: mempool 页可查询 `accepted/pending` 交易与统计计数。
  - SC-6: 启动器面板新增 Address/Contracts/Assets/Mempool 视图，native/web 行为一致。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要按账户查看余额与链上状态。
  - 测试/运维人员：需要快速定位 pending 交易与系统合约能力边界。
  - 启动器维护者：需要在现有 explorer 架构上增量扩展可观测能力。
- User Scenarios & Frequency:
  - 地址排障：每次转账后 1~3 次。
  - 发布巡检：每个候选版本至少 1 次（覆盖地址、资产、mempool）。
  - 功能核验：新链能力上线前后各 1 次（合约/资产能力变更）。
- User Stories:
  - PRD-WORLD_SIMULATOR-026: As a 启动器玩家, I want address/contract/asset/mempool views in explorer, so that I can inspect public-chain states from one panel.
- Critical User Flows:
  1. Flow-LAUNCHER-EXPLORER-P1-001（地址页）:
     `输入 account_id -> 查询地址详情 -> 查看余额/nonce/交易列表 -> 翻页`
  2. Flow-LAUNCHER-EXPLORER-P1-002（合约页）:
     `打开 Contracts -> 浏览系统合约目录 -> 输入 contract_id 查看详情`
  3. Flow-LAUNCHER-EXPLORER-P1-003（资产页）:
     `打开 Assets -> 查看主 token 供应与账户持仓 -> 观察 NFT 能力状态`
  4. Flow-LAUNCHER-EXPLORER-P1-004（内存池）:
     `打开 Mempool -> 筛选 accepted/pending -> 翻页定位待确认交易`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Address RPC | `account_id`、`balance`、`last_nonce`、`next_nonce_hint`、`txs(limit/cursor)` | 输入账户点击查询 `/api/chain/explorer/address` | `idle -> loading -> ready/failed` | 交易按 `submitted_at desc + tx_hash desc` | 只读 |
| Contracts RPC | `items[]`（`contract_id/type/status/summary`） | 打开页签或刷新时请求 `/api/chain/explorer/contracts` | `idle -> loading -> ready/failed` | 默认按 `contract_id asc` | 只读 |
| Contract RPC | `contract_id`、`metadata`、`recent_txs` | 输入 `contract_id` 请求 `/api/chain/explorer/contract` | `idle -> loading -> ready/failed` | 交易按 `submitted_at desc` | 只读 |
| Assets RPC | `token_symbol/decimals/supply`、`holders[]`、`nft_supported` | 打开页签或过滤 owner 请求 `/api/chain/explorer/assets` | `idle -> loading -> ready/failed` | 持仓按 `liquid_balance desc + account_id asc` | 只读 |
| Mempool RPC | `status_filter`、`items[]`、`accepted_count`、`pending_count` | 打开页签、筛选、翻页请求 `/api/chain/explorer/mempool` | `idle -> loading -> ready/failed` | 交易按 `submitted_at desc + tx_hash desc` | 只读 |
| 启动器浏览器 P1 面板 | `tab(address/contracts/assets/mempool)`、`filters`、`page_cursor` | 切换 tab、查询、翻页、详情跳转 | `closed/open` + 子状态机 | 字段/文案在 native/web 保持一致 | 链就绪可操作 |
- Acceptance Criteria:
  - AC-1: runtime 可处理 `GET /v1/chain/explorer/address/contracts/contract/assets/mempool`。
  - AC-2: `oasis7_web_launcher` 提供对应 `/api/chain/explorer/*` 代理路由。
  - AC-3: 地址查询支持账户输入校验与分页，`not_found` 保持结构化语义。
  - AC-4: 合约目录与单合约详情可查询，且明确“系统合约”边界。
  - AC-5: 资产页返回主 token 供应、holders 列表与 `nft_supported` 状态字段。
  - AC-6: mempool 支持 `accepted/pending/all` 筛选、分页与计数统计。
  - AC-7: 启动器 explorer 面板新增 Address/Contracts/Assets/Mempool 四视图。
  - AC-8: 对应任务、测试可追溯到 `PRD-WORLD_SIMULATOR-026`。
- Non-Goals:
  - 不实现 EVM 字节码反编译、事件 ABI 自动解码。
  - 不实现 NFT 元数据抓取与外部图片渲染。
  - 不引入跨链桥、跨分片聚合或链外索引服务。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - 运行时层: 在 explorer store 基础上增加 address/contracts/assets/mempool 查询聚合。
  - 控制面层: `oasis7_web_launcher` 新增 P1 路由 remap 到 runtime。
  - 客户端层: 扩展 explorer 窗口状态机与请求层，新增四个视图并复用统一事件通道。
- Integration Points:
  - `crates/oasis7/src/bin/oasis7_chain_runtime/explorer_p0_api.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api_tests.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/oasis7_web_launcher_tests.rs`
  - `crates/oasis7_client_launcher/src/app_process.rs`
  - `crates/oasis7_client_launcher/src/app_process_web.rs`
  - `crates/oasis7_client_launcher/src/explorer_window.rs`
- Edge Cases & Error Handling:
  - `account_id/contract_id` 为空时返回 `invalid_request`。
  - 地址不存在且无交易历史时返回 `not_found`。
  - `mempool status` 非法值时返回 `invalid_request`。
  - 资产页 owner 过滤账户不存在时返回空列表而非 panic。
  - 当前链不支持 NFT 时返回 `nft_supported=false` + 空集合。
  - 执行世界读取失败时返回结构化 `internal_error`，不影响 runtime 进程。
- Non-Functional Requirements:
  - NFR-1: 默认分页 50 条下，P1 查询本地链路 `p95 <= 500ms`。
  - NFR-2: native/web 视图交互一致率 100%。
  - NFR-3: 单 Rust 文件保持 < 1200 行（必要时拆分模块）。
  - NFR-4: 现有 P0 能力与转账/反馈链路无回归。
- Security & Privacy:
  - 查询接口仅返回公开链状态字段。
  - 错误消息保留可诊断性并避免泄露本地敏感路径。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成 PRD 与任务拆解。
  - v1.1: 完成 runtime + 控制面 P1 API。
  - v2.0: 完成启动器 P1 UI 与跨端回归。
- Technical Risks:
  - 风险-1: 执行世界读盘失败会导致地址/资产数据不可用。
  - 风险-2: explorer store 过滤逻辑扩大后，分页性能可能退化。
  - 风险-3: 启动器单文件膨胀超过 1200 行需拆分。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-026 -> TASK-WORLD_SIMULATOR-060/061/062 -> `test_tier_required`。
  - 计划验证命令:
    - `./scripts/doc-governance-check.sh`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_chain_runtime transfer_submit_api::tests:: -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p oasis7_client_launcher --target wasm32-unknown-unknown`
- Decision Log:
  - DEC-LAUNCHER-EXPLORER-P1-001: 合约页先以“系统合约目录 + 单合约详情”落地，不引入泛化智能合约执行语义。理由：当前链 runtime 以系统能力为主，先保证可观测闭环。
  - DEC-LAUNCHER-EXPLORER-P1-002: 资产页先覆盖主 token 与 NFT 能力状态位（`nft_supported`），后续再扩展 NFT 索引。理由：符合当前链能力边界且可平滑扩展。
  - DEC-LAUNCHER-EXPLORER-P1-003: mempool 以 `accepted/pending` 视图表达未最终确认交易。理由：可直接复用现有生命周期模型，实施成本低且诊断价值高。
