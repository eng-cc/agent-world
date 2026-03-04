# 客户端启动器 Web wasm 时间兼容与闭环修复（2026-03-04）项目管理文档

- 对应设计文档: doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 修复 launcher wasm 时间兼容问题（移除不支持的平台时间调用），并通过 wasm 编译验证。
- [ ] T2 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 执行 Playwright headed 闭环（open/snapshot/console/screenshot + `/api/state`）并归档证据。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/Cargo.toml`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `output/playwright/`

## 状态
- 当前阶段: in_progress
- 当前任务: T1
- 备注: 本专题聚焦“launcher wasm 启动 panic(time not implemented)”阻断问题，目标是修复并形成可复现闭环证据。
