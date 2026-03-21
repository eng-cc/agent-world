# 客户端启动器区块链浏览器公共主链视角 P0（2026-03-07）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.project.md`

审计轮次: 6

## 1. Executive Summary
- Problem Statement: 当前启动器浏览器仅覆盖 `overview/transactions/transaction(action_id)`，缺少区块分页、`tx_hash` 视图、统一搜索与重启可保留的索引，不满足“公共主链浏览器”最小可观测能力。
- Proposed Solution: 在 runtime 增加持久化 explorer 索引与 P0 查询接口（blocks/block/txs/tx/search），控制面代理透传，并在 native/web 共用启动器面板补齐区块列表、交易哈希详情、搜索与分页交互。
- Success Criteria:
  - SC-1: `oasis7_chain_runtime` 提供 `blocks/block/txs/tx/search` 五类 explorer P0 查询接口。
  - SC-2: 交易详情支持 `tx_hash` 查询并返回 receipt-like 字段（`status/block_height/block_hash/action_count/timestamps`）。
  - SC-3: 区块列表、交易列表均支持分页参数，排序稳定且可重复翻页。
  - SC-4: explorer 索引在 runtime 重启后可恢复（至少保留最近窗口数据）。
  - SC-5: 启动器面板支持区块浏览、交易哈希详情、统一搜索与分页，native/web 行为一致。
  - SC-6: `test_tier_required` 回归通过，且不回归既有转账/反馈/状态链路。

## 2. User Experience & Functionality
- User Personas:
  - 启动器玩家：需要快速确认区块推进、交易上链与最终状态。
  - 测试/运维人员：需要在 GUI 内搜索 `height/hash/action/account`，无需命令行排障。
  - 启动器维护者：需要跨端同源前端与可持续扩展的浏览器 API。
- User Scenarios & Frequency:
  - 发布前链稳定性巡检：每个候选版本至少 1 次（中频）。
  - 提交转账后定位交易：每笔交易后 1 次（高频）。
  - 故障定位（区块停滞/交易丢失）：发生时连续多次（高频突发）。
- User Stories:
  - PRD-WORLD_SIMULATOR-025: As a 启动器玩家, I want block/tx/search pagination in explorer, so that I can inspect chain state like a public-chain browser.
- Critical User Flows:
  1. Flow-LAUNCHER-EXPLORER-P0-001（区块分页）:
     `打开浏览器面板 -> 选择 Blocks -> 查看最新区块列表 -> 翻页 -> 点击区块查看详情`
  2. Flow-LAUNCHER-EXPLORER-P0-002（交易哈希详情）:
     `打开 Txs -> 输入或点击 tx_hash -> 查看 receipt-like 明细（状态/区块/时间/错误）`
  3. Flow-LAUNCHER-EXPLORER-P0-003（统一搜索）:
     `输入关键词（height/block_hash/tx_hash/action_id/account_id）-> 返回分类结果 -> 跳转详情`
  4. Flow-LAUNCHER-EXPLORER-P0-004（重启恢复）:
     `runtime 重启 -> explorer 索引恢复 -> 列表/详情仍可查询最近历史`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Explorer Blocks RPC | `limit`、`cursor(offset)`、`items[]`、`next_cursor`、`total` | 打开 Blocks 或翻页时请求 `/api/chain/explorer/blocks` | `idle -> loading -> ready/failed` | 默认 `height desc`，同高度按 `block_hash` 稳定排序 | 只读 |
| Explorer Block RPC | `height/hash`、`action_count`、`committed_at`、`tx_hashes[]` | 点击区块行或输入 `height/hash` 请求 `/api/chain/explorer/block` | `idle -> loading -> ready/failed` | 单区块精确命中；`height` 与 `hash` 二选一 | 只读 |
| Explorer Txs RPC | `account_filter`、`status_filter`、`limit`、`cursor(offset)`、`items[]`、`next_cursor` | 打开 Txs、过滤、翻页请求 `/api/chain/explorer/txs` | `idle -> loading -> ready/failed` | 默认 `submitted_at desc + tx_hash desc` | 只读 |
| Explorer Tx RPC | `tx_hash`、`action_id`、`status`、`block_height/hash`、`error` | 输入 `tx_hash` 或列表点击请求 `/api/chain/explorer/tx` | `idle -> loading -> ready/failed` | `tx_hash` 精确查询 | 只读 |
| Explorer Search RPC | `q`、`types[]`、`items[]`（block/tx/action/account） | 输入关键词点击搜索请求 `/api/chain/explorer/search` | `idle -> loading -> ready/failed` | 先 exact 再 prefix；分类返回 | 只读 |
| Explorer 持久化索引 | `index_version`、`last_synced_height`、`blocks[]`、`txs[]` | runtime 启动加载；每次同步后写盘 | `cold_start -> loaded -> syncing -> persisted` | 仅保留最近窗口（如 5k tx / 2k blocks） | 本地文件可写 |
| 启动器浏览器 P0 面板 | `tab(blocks/txs/search)`、`page_cursor`、`filters`、`detail_card` | 切换 Tab、翻页、搜索、详情跳转 | `closed/open` + 子状态机 | UI 与 web/native 文案与字段一致 | 链就绪可操作 |
- Acceptance Criteria:
  - AC-1: `oasis7_chain_runtime` 支持 `GET /v1/chain/explorer/blocks`、`/block`、`/txs`、`/tx`、`/search`。
  - AC-2: `oasis7_web_launcher` 提供对应 `/api/chain/explorer/*` 代理并保留结构化错误语义。
  - AC-3: 区块与交易接口支持分页参数 `limit/cursor`，默认 `limit=50`，`limit` 上限可控（<=200）。
  - AC-4: `tx` 详情支持 `tx_hash` 查询，返回 receipt-like 结果；`not_found` 保持结构化错误。
  - AC-5: 搜索支持 `height/block_hash/tx_hash/action_id/account_id`，返回可区分类型的结果集合。
  - AC-6: 索引数据重启后恢复最近窗口，且损坏文件可回退空索引并继续服务。
  - AC-7: 启动器面板新增 Blocks/Txs/Search 视图与分页控件，native/web 同源。
  - AC-8: 对应任务与测试可追溯到 `PRD-WORLD_SIMULATOR-025`。
- Non-Goals:
  - 不在本轮实现 Merkle 证明、交易回执加密证明或跨链桥浏览。
  - 不引入地址标签系统、复杂图表分析、告警订阅中心。
  - 不改造共识/执行语义，仅补查询索引与展示能力。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属能力。

## 4. Technical Specifications
- Architecture Overview:
  - 运行时层: 新增 explorer 索引模块，统一消费 committed batches，维护内存 + 本地持久化索引，并提供 P0 查询接口。
  - 控制面层: `oasis7_web_launcher` 透传 explorer P0 查询到 runtime，保持统一错误封装。
  - 客户端层: 启动器 explorer 窗口扩展为 Blocks/Txs/Search 三视图，并增加分页与详情联动。
- Integration Points:
  - `crates/oasis7/src/bin/oasis7_chain_runtime.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api_tests.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/explorer_p0_api.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/oasis7_web_launcher_tests.rs`
  - `crates/oasis7_client_launcher/src/app_process.rs`
  - `crates/oasis7_client_launcher/src/app_process_web.rs`
  - `crates/oasis7_client_launcher/src/explorer_window.rs`
- Edge Cases & Error Handling:
  - `limit/cursor` 非法时返回 `invalid_request`（400）并给出字段级错误文案。
  - `block` 查询同时缺失 `height/hash` 时拒绝；两者同时提供按 `height` 优先并记录提示。
  - `tx_hash` 非法格式或不存在时返回 `not_found`，不触发 panic。
  - 搜索词为空或超长时返回 `invalid_request`；超长阈值固定（如 128）。
  - 索引文件不存在时自动初始化；损坏时记录日志并重建空索引。
  - 索引写盘失败时保持内存可读并返回 `internal_error` 到日志，不阻断读取请求。
  - 链未就绪或代理不可达时维持 `chain_disabled/proxy_error` 结构化语义。
- Non-Functional Requirements:
  - NFR-1: `blocks/txs/tx/block/search` 本地链路 `p95 <= 500ms`（默认分页 50）。
  - NFR-2: 分页翻页操作端到端可见延迟 <= 1s（本地网络）。
  - NFR-3: 持久化索引写盘采用原子替换，避免中断后半文件状态。
  - NFR-4: 索引窗口默认保留最近 `<= 5000` 交易与 `<= 2000` 区块，内存占用可控。
  - NFR-5: 新增 Rust 文件保持单文件 < 1200 行。
  - NFR-6: native/web 浏览器面板字段、按钮、错误语义一致率 100%。
- Security & Privacy:
  - 查询接口仅输出公开账本字段，不暴露私钥、签名材料或本地敏感路径。
  - 错误消息保持可诊断但需脱敏（不泄露本地绝对路径与凭据）。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成专题 PRD 建模与任务拆解。
  - v1.1: 落地 runtime explorer P0 API + 持久化索引 + 控制面代理。
  - v2.0 (2026-03-07): 落地启动器 Blocks/Txs/Search 分页 UI 并完成跨端回归。
- Technical Risks:
  - 风险-1: committed batch 为单消费语义，若多处独立 drain 会导致索引/历史漂移。
  - 风险-2: 索引写盘频率过高可能导致 I/O 抖动。
  - 风险-3: 搜索范围扩大后若未做窗口上限，内存与响应时延会退化。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-025 -> TASK-WORLD_SIMULATOR-057/058/059 -> `test_tier_required`。
  - 计划验证命令:
    - `./scripts/doc-governance-check.sh`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_chain_runtime transfer_submit_api::tests:: -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_web_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p oasis7_client_launcher -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo check -p oasis7_client_launcher --target wasm32-unknown-unknown`
- Decision Log:
  - DEC-LAUNCHER-EXPLORER-P0-001: 采用统一 explorer store 消费 committed batches（单点 drain），旧/新查询接口共享该状态源。理由：避免多消费者竞争导致的批次丢失与查询漂移。
  - DEC-LAUNCHER-EXPLORER-P0-002: P0 分页采用 `cursor(offset)` 而非复杂游标签名。理由：先满足可翻页可重复读取，再在后续版本升级强一致游标。
  - DEC-LAUNCHER-EXPLORER-P0-003: 持久化索引采用本地 JSON 文件 + 原子替换。理由：实现成本低、可跨平台、便于调试与恢复。
