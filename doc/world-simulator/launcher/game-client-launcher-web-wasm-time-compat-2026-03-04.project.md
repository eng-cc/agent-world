# 客户端启动器 Web wasm 时间兼容与闭环修复（2026-03-04）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.md`

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 修复 launcher wasm 时间兼容问题（移除不支持的平台时间调用），并通过 wasm 编译验证。
- [x] T2 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 执行 Playwright headed 闭环（open/snapshot/console/screenshot + `/api/state`）并归档证据。
- [x] T3 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 修复 `self_guided` wasm 时间回归（移除 `SystemTime::now` 触发点）并完成启动器 Web 启停闭环复验（要求 `time not implemented` 零命中）。

## 依赖
- `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.design.md`
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/Cargo.toml`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `output/playwright/`

## 状态
- 最近更新：2026-03-09（回归修复任务 T3 已完成并归档）
- 当前阶段: done
- 当前任务: 无
- 备注: 历史修复（T0~T3）已完成，回归证据目录 `output/playwright/launcher-web-ui-timefix-20260309-113010/`；控制台已验证无 `time not implemented` / `RuntimeError: unreachable` 签名。
