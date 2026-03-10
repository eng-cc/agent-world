# 启动器区块链浏览器公共主链 P0 设计（2026-03-07）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.project.md`

## 1. 设计定位
定义 explorer P0 在 runtime、控制面与 launcher UI 三层之间的分页查询、哈希详情、统一搜索与重启恢复索引结构，使启动器具备公共主链浏览器的最小可观测能力。

## 2. 设计结构
- 运行时索引层：持久化维护 blocks/block/txs/tx/search 所需的最近窗口索引。
- 控制面代理层：`world_web_launcher` 透传 `/api/chain/explorer/*` P0 查询接口并保留结构化错误。
- UI 视图层：Blocks、Txs、Search 三视图共享分页、跳转和详情状态机。
- 搜索编排层：对 `height/block_hash/tx_hash/action_id/account_id` 做统一解析与分类跳转。

## 3. 关键接口 / 入口
- `/v1/chain/explorer/blocks`
- `/v1/chain/explorer/block`
- `/v1/chain/explorer/txs`
- `/v1/chain/explorer/tx`
- `/v1/chain/explorer/search`
- `/api/chain/explorer/*`
- `agent_world_client_launcher/src/explorer_window.rs`

## 4. 约束与边界
- 区块与交易列表排序必须稳定，支持可重复翻页。
- `tx_hash` 详情需返回 receipt-like 字段，但不扩展写操作。
- 重启恢复只要求保留最近窗口数据，不承诺完整历史归档。
- 本阶段不覆盖地址/合约/资产/内存池等 P1 能力。

## 5. 设计演进计划
- 先冻结 P0 查询契约与分页模型。
- 再补 runtime 索引与控制面代理。
- 最后落地 native/web 共用 UI 的区块、交易与搜索交互。
