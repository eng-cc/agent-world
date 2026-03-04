# 客户端启动器 Web 必填配置校验分流修复（2026-03-04）项目管理文档

- 对应设计文档: doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-014) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-014) [test_tier_required]: 修复 launcher Web 必填校验分流（移除 wasm 场景下 binary path 必填阻断）。
- [ ] T2 (PRD-WORLD_SIMULATOR-014) [test_tier_required]: 修复 launcher Web 字段渲染分流（按 web_visible 渲染）并完成 Playwright headed 闭环回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/launcher_core.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_launcher_ui/src/lib.rs`
- `output/playwright/`

## 状态
- 当前阶段: in_progress
- 当前任务: T1
- 备注: 目标是消除 Web 端 native-only 必填项误报，同时保持 native 校验不退化。
