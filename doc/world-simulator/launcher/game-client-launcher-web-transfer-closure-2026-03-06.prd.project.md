# 客户端启动器 Web 链上转账闭环补齐（2026-03-06）项目管理文档

审计轮次: 5
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-020) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-020) [test_tier_required]: 落地 Web 转账闭环（`/api/chain/transfer` 代理 + wasm 转账窗口提交 + 回归测试）。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher/control_plane.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/src/transfer_window_web.rs`
- `crates/agent_world/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-06
- 当前阶段: in_progress
- 当前任务: T1
- 备注: 先完成文档建模；代码闭环与测试证据由 T1 收口。
