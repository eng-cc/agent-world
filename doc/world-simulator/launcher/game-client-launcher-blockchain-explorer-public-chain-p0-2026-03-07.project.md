# 客户端启动器区块链浏览器公共主链视角 P0（2026-03-07）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.prd.md`

审计轮次: 6

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-025) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-025) [test_tier_required]: 落地 `world_chain_runtime` explorer P0 API（blocks/block/txs/tx/search）与持久化索引，并补齐 runtime/控制面契约测试。
- [x] T2 (PRD-WORLD_SIMULATOR-025) [test_tier_required]: 扩展启动器 explorer 面板（Blocks/Txs/Search + 分页 + tx_hash 详情）并完成 native/web 回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/oasis7/src/bin/world_chain_runtime.rs`
- `crates/oasis7/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `crates/oasis7/src/bin/world_chain_runtime/transfer_submit_api_tests.rs`
- `crates/oasis7/src/bin/world_chain_runtime/explorer_p0_api.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher/oasis7_web_launcher_tests.rs`
- `crates/oasis7_client_launcher/src/app_process.rs`
- `crates/oasis7_client_launcher/src/app_process_web.rs`
- `crates/oasis7_client_launcher/src/explorer_window.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-07
- 当前阶段: completed
- 当前任务: 无
- 备注: T0/T1/T2 已完成（runtime + 控制面代理 + 启动器跨端 UI 收口 + 回归）。
