# 客户端启动器转账产品级体验与跨端同层前端（2026-03-06）项目管理文档

审计轮次: 5
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-023) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-023) [test_tier_required]: 扩展控制面/链运行时转账查询能力（余额辅助、状态查询、历史查询 API）并完成契约测试。
- [ ] T2 (PRD-WORLD_SIMULATOR-023) [test_tier_required]: 将 native/web 转账窗口收敛为同一套前端实现（同层复用），补齐账户选择、自动 nonce、最终状态与历史视图。
- [ ] T3 (PRD-WORLD_SIMULATOR-023) [test_tier_required + test_tier_full]: 完成跨端回归（native + wasm + control plane + runtime）与手册/证据收口。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/src/transfer_window.rs`
- `crates/agent_world_client_launcher/src/transfer_window_web.rs`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `crates/agent_world/src/bin/world_chain_runtime/balances_api.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-06
- 当前阶段: in_progress
- 当前任务: T1（转账查询能力与 API 契约扩展）
- 备注: T0 已完成（文档建档与主文档树回写）；后续按 T1->T2->T3 推进产品级能力落地。
