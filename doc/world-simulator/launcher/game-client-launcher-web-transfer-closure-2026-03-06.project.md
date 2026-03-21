# 客户端启动器 Web 链上转账闭环补齐（2026-03-06）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.prd.md`

审计轮次: 6

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-020) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-020) [test_tier_required]: 落地 Web 转账闭环（`/api/chain/transfer` 代理 + wasm 转账窗口提交 + 回归测试）。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/oasis7/src/bin/oasis7_web_launcher.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher/control_plane.rs`
- `crates/oasis7_client_launcher/src/main.rs`
- `crates/oasis7_client_launcher/src/app_process_web.rs`
- `crates/oasis7_client_launcher/src/transfer_window.rs`
- `crates/oasis7/src/bin/world_chain_runtime/transfer_submit_api.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-06
- 当前阶段: completed
- 当前任务: 无（T0/T1 已完成）
- 备注: Web 转账入口已在 wasm 界面可用，控制面新增 `/api/chain/transfer` 代理并完成 required 回归。
