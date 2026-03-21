# 客户端启动器区块链浏览器面板设计（2026-03-07）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-panel-2026-03-07.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-panel-2026-03-07.project.md`

## 1. 设计定位
定义启动器内“区块链浏览器”面板的跨端统一结构：由 runtime 提供 explorer 查询 RPC，经 `oasis7_web_launcher` 代理后，由 native/web 共用 UI 展示链总览、交易列表与交易详情。

## 2. 设计结构
- 运行时查询层：`oasis7_chain_runtime` 暴露 overview、transactions、transaction 三类只读 explorer RPC。
- 控制面代理层：`oasis7_web_launcher` 将 explorer RPC 统一代理到 `/api/chain/explorer/*`，保留结构化错误语义。
- 启动器展示层：`oasis7_client_launcher` 提供 explorer window，复用现有生命周期状态枚举与轮询机制。
- 交互状态层：overview、filters、transactions、`selected_transaction` 分别维护 `idle/loading/ready/failed` 状态。

## 3. 关键接口 / 入口
- `GET /v1/chain/explorer/overview`
- `GET /v1/chain/explorer/transactions`
- `GET /v1/chain/explorer/transaction`
- `GET /api/chain/explorer/overview`
- `GET /api/chain/explorer/transactions`
- `GET /api/chain/explorer/transaction`
- `crates/oasis7_client_launcher/src/explorer_window.rs`

## 4. 约束与边界
- native/web 必须共用同一控制面协议，避免字段与错误语义分叉。
- explorer 仅暴露公共账本字段，不返回私钥、凭据或本地敏感配置。
- 非法 `status` / `action_id` 必须返回结构化错误并允许用户继续重试。
- 本轮只补齐查询与展示能力，不覆盖区块分页、Merkle 证明、钱包管理或跨链能力。

## 5. 设计演进计划
- 先冻结 explorer RPC 契约与代理路径。
- 再接入 launcher 面板、过滤器和详情查询交互。
- 最后完成 native/web 同源回归，确保刷新、错误态和明细查看行为一致。
