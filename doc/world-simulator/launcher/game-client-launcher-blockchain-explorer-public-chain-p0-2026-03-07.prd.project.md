# 客户端启动器区块链浏览器公共主链视角 P0（2026-03-07）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-025) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-025) [test_tier_required]: 落地 `world_chain_runtime` explorer P0 API（blocks/block/txs/tx/search）与持久化索引，并补齐 runtime/控制面契约测试。
- [ ] T2 (PRD-WORLD_SIMULATOR-025) [test_tier_required]: 扩展启动器 explorer 面板（Blocks/Txs/Search + 分页 + tx_hash 详情）并完成 native/web 回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api_tests.rs`
- `crates/agent_world/src/bin/world_chain_runtime/explorer_p0_api.rs`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/src/explorer_window.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-07
- 当前阶段: in_progress
- 当前任务: T2（启动器 explorer 面板 P0 扩展）
- 备注: T0/T1 已完成（runtime + 控制面代理 + 契约测试），后续执行 T2 并收口跨端回归。
