# 客户端启动器区块链浏览器公共主链视角 P1（地址/合约/资产/内存池，2026-03-08）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.prd.md`

审计轮次: 6

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-026) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-026) [test_tier_required]: 落地 runtime + 控制面 explorer P1 API（address/contracts/contract/assets/mempool）并补齐契约测试。
- [x] T2 (PRD-WORLD_SIMULATOR-026) [test_tier_required]: 扩展启动器 explorer P1 面板（Address/Contracts/Assets/Mempool）并完成 native/web 回归。

## 依赖
- `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p1-address-contract-assets-mempool-2026-03-08.design.md`
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/oasis7/src/bin/world_chain_runtime.rs`
- `crates/oasis7/src/bin/world_chain_runtime/explorer_p0_api.rs`
- `crates/oasis7/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `crates/oasis7/src/bin/world_chain_runtime/transfer_submit_api_tests.rs`
- `crates/oasis7/src/bin/world_web_launcher.rs`
- `crates/oasis7/src/bin/world_web_launcher/world_web_launcher_tests.rs`
- `crates/oasis7_client_launcher/src/app_process.rs`
- `crates/oasis7_client_launcher/src/app_process_web.rs`
- `crates/oasis7_client_launcher/src/explorer_window.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-08
- 当前阶段: completed
- 当前任务: 无
- 备注: T0/T1/T2 已完成，`PRD-WORLD_SIMULATOR-026` 交付闭环。
