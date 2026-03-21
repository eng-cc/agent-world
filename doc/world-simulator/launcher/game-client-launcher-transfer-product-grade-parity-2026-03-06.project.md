# 客户端启动器转账产品级体验与跨端同层前端（2026-03-06）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.prd.md`

审计轮次: 6

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-023) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-023) [test_tier_required]: 扩展控制面/链运行时转账查询能力（余额辅助、状态查询、历史查询 API）并完成契约测试。
- [x] T2 (PRD-WORLD_SIMULATOR-023) [test_tier_required]: 将 native/web 转账窗口收敛为同一套前端实现（同层复用），补齐账户选择、自动 nonce、最终状态与历史视图。
- [x] T3 (PRD-WORLD_SIMULATOR-023) [test_tier_required + test_tier_full]: 完成跨端回归（native + wasm + control plane + runtime）与手册/证据收口。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/oasis7_client_launcher/src/main.rs`
- `crates/oasis7_client_launcher/src/app_process.rs`
- `crates/oasis7_client_launcher/src/app_process_web.rs`
- `crates/oasis7_client_launcher/src/transfer_window.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher/transfer_query_proxy.rs`
- `crates/oasis7/src/bin/oasis7_chain_runtime.rs`
- `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api.rs`
- `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api_tests.rs`
- `crates/oasis7/src/runtime/world/resources.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-07
- 当前阶段: completed
- 当前任务: 无（T0/T1/T2/T3 已完成）
- 备注: 已完成 runtime 查询 API（accounts/status/history）、控制面代理、shared transfer panel 与跨端回归；本专题进入维护态。
