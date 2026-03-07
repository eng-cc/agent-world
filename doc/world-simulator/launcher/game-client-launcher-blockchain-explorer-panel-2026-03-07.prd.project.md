# 客户端启动器区块链浏览器面板（2026-03-07）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-panel-2026-03-07.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-024) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-024) [test_tier_required]: 落地 `world_chain_runtime` explorer RPC 与 `world_web_launcher` 代理接口，并补齐契约测试。
- [ ] T2 (PRD-WORLD_SIMULATOR-024) [test_tier_required]: 落地启动器“区块链浏览器”面板（native/web 同源）并完成跨端回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api_tests.rs`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher/world_web_launcher_tests.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/src/explorer_window.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-07
- 当前阶段: in_progress
- 当前任务: T1（RPC/代理接口补齐）
- 备注: T0 文档建模已完成，按“先 RPC 再 UI”顺序推进。
